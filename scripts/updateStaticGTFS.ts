import {mkdir, readFile} from 'node:fs/promises';
import {Readable} from 'node:stream';
import {pipeline} from 'node:stream/promises';
import type {ReadableStream as NodeReadableStream} from 'node:stream/web';
import * as unzipper from 'unzipper';
import {MongoClient} from 'mongodb';
import * as dotenv from 'dotenv';
import {parse} from "csv-parse/sync"
import {
    S3Client,
    PutObjectCommand,
} from "@aws-sdk/client-s3";
import {promisify} from "node:util";
import {gzip} from "node:zlib";

dotenv.config({ path: './.env.local' });

const DATA_VIC_STATIC_GTFS_URL = "https://opendata.transport.vic.gov.au/dataset/3f4e292e-7f8a-4ffe-831f-1953be0fe448/resource/fb152201-859f-4882-9206-b768060b50ad/download/gtfs.zip";
const DATA_OUTPUT_DIR = "./staticData";
const client = new MongoClient(process.env.MONGODB_URI || "")
const s3 = new S3Client({
    region: "auto",
    endpoint: process.env.R2_S3_ENDPOINT,
    credentials: {
        accessKeyId: process.env.R2_ACCESS_KEY_ID!,
        secretAccessKey: process.env.R2_SECRET_ACCESS_KEY!,
    },
});
const gzipAsync = promisify(gzip);

interface Agency {
    agency_id: number,
    agency_name: string,
    agency_url: string,
    agency_timezone: string,
    agency_lang: string,
    agency_fare_url: string,
}

interface CalendarDates {
    service_id: string,
    date: string,
    exception_type: "1" | "2",
}

interface Calendar {
    service_id: string,
    monday: "0" | "1",
    tuesday: "0" | "1",
    wednesday: "0" | "1",
    thursday: "0" | "1",
    friday: "0" | "1",
    saturday: "0" | "1",
    sunday: "0" | "1",
    start_date: string,
    end_date: string
}

interface Levels {
    level_id: string,
    level_index: string,
    level_name: string
}

interface Pathways {
    pathway_id: string,
    from_stop_id: string,
    to_stop_id: string,
    pathway_mode: "1" | "2" | "3" | "4" | "5",
    is_bidirectional: "0" | "1",
    traversal_time: number
}

interface Routes {
    route_id: string,
    agency_id: string,
    route_short_name: string,
    route_long_name: string,
    route_type: "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "10" | "11" | "12",
    route_color: string,
    route_text_color: string
}

interface Shapes {
    shape_id: string,
    shape_pt_lat: number,
    shape_pt_lon: number,
    shape_pt_sequence: number,
    shape_dist_traveled: number
}

interface StopTimes {
    trip_id: string,
    arrival_time: string,
    departure_time: string,
    stop_id: string,
    stop_sequence: number,
    stop_headsign: string,
    pickup_type: "0" | "1" | "2" | "3",
    drop_off_type: "0" | "1" | "2" | "3",
    shape_dist_traveled: "0" | "1" | "2" | "3"
}

interface Stops {
    stop_id: string,
    stop_name: string,
    stop_lat: number,
    stop_lon: number,
    stop_url: string,
    location_type: "0" | "1" | "2" | "3" | "4",
    parent_station: string,
    wheelchair_boarding: "0" | "1" | "2",
    level_id: string
}

interface Transfers {
    from_stop_id: string,
    to_stop_id: string,
    from_route_id: string,
    to_route_id: string,
    from_trip_id: string,
    to_trip_id: string,
    transfer_type: "0" | "1" | "2" | "3" | "4" | "5",
    min_transfer_time: number
}

interface Trips {
    route_id: string,
    service_id: string,
    trip_id: string,
    shape_id: string,
    trip_headsign: string,
    direction_id: "0" | "1",
    block_id: string,
    wheelchair_accessible: "0" | "1" | "2"
}

const numberFields = [
    "min_transfer_time",
    "stop_lat",
    "stop_lon",
    "stop_sequence",
    "shape_pt_lat",
    "shape_pt_lon",
    "shape_pt_sequence",
    "shape_dist_traveled",
    "traversal_time"
]

// 1 (Regional Train)
// 2 (Metropolitan Train)
// 3 (Metropolitan Tram)
// 4 Myki Bus (Metro Bus and Regional Town Bus)
// 5 (Regional Coach)
// 6 (Regional Bus)
// 10 (Interstate)
// 11 (SkyBus)
interface GTFSInfo {
    type: 1 | 2 | 3 | 4 | 5 | 6 | 10 | 11,
    agency: Agency[],
    calendar: Calendar[],
    calendarDates: CalendarDates[],
    levels: Levels[],
    pathways: Pathways[],
    routes: Routes[],
    shapes: Shapes[],
    stopTimes: StopTimes[],
    stops: Stops[],
    transfers: Transfers[],
    trips: Trips[],
}

async function connectToMongoDB() {
    try {
        await client.connect();
        console.log("Connected to MongoDB");
        return client;
    } catch (err) {
        console.dir(err);
    }
}

async function downloadAndExtract(
    url: string,
    outputDir: string,
) {
    console.log("Downloading...")

    const response = await fetch(url);

    if (!response.ok || !response.body) {
        throw new Error(`Download failed: HTTP ${response.status}`);
    }

    await mkdir(outputDir, { recursive: true });

    await pipeline(
        Readable.fromWeb(response.body as unknown as NodeReadableStream),
        unzipper.Extract({ path: outputDir }),
    );
}

async function extractFurtherFiles(dataDir: string) {
    console.log("Extracting...")

    for (let i = 1; i <= 11; i++) {
        if (i >= 7 && i <= 9) continue;

        const zip = await unzipper.Open.file(dataDir + `/${i}/google_transit.zip`);
        await zip.extract({ path: dataDir + `/${i}` })
    }
}

async function parseCSV<T>(fileName: string) {
    const csv = await readFile(fileName, "utf-8");
    return parse(csv, {
        columns: true,
        skip_empty_lines: true,
        cast: (value, context) => {
            if (numberFields.includes(context.column.toString())) {
                return Number(value);
            }

            return value;
        },
    }) as T[];
}

async function uploadInBatches<T extends object>(
    collectionName: string,
    type: number,
    records: T[],
    batchSize = 1000,
) {
    const collection = client
        .db("gtfs")
        .collection(collectionName);

    await collection.deleteMany({});

    for (let i = 0; i < records.length; i += batchSize) {
        const batch = records.slice(i, i + batchSize);

        await collection.insertMany(
            batch.map(record => ({ ...record, type }))
        );
    }
}

async function uploadGzippedFile(
    filePath: string,
    key: string,
) {
    const data = await readFile(filePath);
    const compressed = await gzipAsync(data);

    await s3.send(
        new PutObjectCommand({
            Bucket: "pt-gtfs",
            Key: key,
            Body: compressed,
            ContentType: "application/gzip",
        }),
    );

    console.log(`Uploaded ${key}`);
}

async function parseThenUpload(dataDir: string) {
    console.log("Parsing/Uploading...");

    await connectToMongoDB();

    try {
        for (let i = 1; i <= 11; i++) {
            // Only trams and metro trains for now
            if (i !== 2 && i !== 3) continue;

            const fileDir = `${dataDir}/${i}/`;

            const agency = await parseCSV<Agency>(
                fileDir + "agency.txt",
            );
            const calendar = await parseCSV<Calendar>(
                fileDir + "calendar.txt",
            );
            const calendarDates = await parseCSV<CalendarDates>(
                fileDir + "calendar_dates.txt",
            );
            const levels = await parseCSV<Levels>(
                fileDir + "levels.txt",
            );
            const pathways = await parseCSV<Pathways>(
                fileDir + "pathways.txt",
            );
            const routes = await parseCSV<Routes>(
                fileDir + "routes.txt",
            );
            const stops = await parseCSV<Stops>(
                fileDir + "stops.txt",
            );
            const transfers = await parseCSV<Transfers>(
                fileDir + "transfers.txt",
            );
            const trips = await parseCSV<Trips>(
                fileDir + "trips.txt",
            );

            await uploadInBatches<Agency>("agency", i, agency);
            console.log(`Uploaded agency (${i})`);

            await uploadInBatches<Calendar>("calendar", i, calendar);
            console.log(`Uploaded calendar (${i})`);

            await uploadInBatches<CalendarDates>(
                "calendar_dates",
                i,
                calendarDates,
            );
            console.log(`Uploaded calendar_dates (${i})`);

            await uploadInBatches<Levels>("levels", i, levels);
            console.log(`Uploaded levels (${i})`);

            await uploadInBatches<Pathways>("pathways", i, pathways);
            console.log(`Uploaded pathways (${i})`);

            await uploadInBatches<Routes>("routes", i, routes);
            console.log(`Uploaded routes (${i})`);

            await uploadInBatches<Stops>("stops", i, stops);
            console.log(`Uploaded stops (${i})`);

            await uploadInBatches<Transfers>("transfers", i, transfers);
            console.log(`Uploaded transfers (${i})`);

            await uploadInBatches<Trips>("trips", i, trips);
            console.log(`Uploaded trips (${i})`);

            // Store large GTFS files in R2 instead of MongoDB.
            await Promise.all([
                uploadGzippedFile(
                    fileDir + "shapes.txt",
                    `gtfs/${i}/shapes.txt.gz`,
                ),
                uploadGzippedFile(
                    fileDir + "stop_times.txt",
                    `gtfs/${i}/stop_times.txt.gz`,
                ),
            ]);
        }
    } finally {
        await client.close();
    }
}

(async ()  => {
    await downloadAndExtract(DATA_VIC_STATIC_GTFS_URL, DATA_OUTPUT_DIR);
    await extractFurtherFiles(DATA_OUTPUT_DIR);
    await parseThenUpload(DATA_OUTPUT_DIR);
})()
