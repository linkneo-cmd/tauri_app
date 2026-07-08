import sharp from 'sharp';
import fs from 'fs';
import path from 'path';

function processSingleImage(inputPath, outputPath, targetWidth, targetHeight, keepRatio, outputFormat, quality) {
    const originalSize = fs.existsSync(inputPath) ? fs.statSync(inputPath).size : 0;

    return new Promise((resolve) => {
        sharp(inputPath)
            .resize({
                width: targetWidth,
                height: targetHeight,
                fit: keepRatio ? sharp.fit.inside : sharp.fit.fill,
            })
            .toFormat(outputFormat, { quality })
            .toFile(outputPath)
            .then(() => {
                const newSize = fs.existsSync(outputPath) ? fs.statSync(outputPath).size : 0;
                resolve({
                    file: inputPath,
                    success: true,
                    error: null,
                    original_size: originalSize,
                    new_size: newSize,
                });
            })
            .catch((err) => {
                resolve({
                    file: inputPath,
                    success: false,
                    error: err.message,
                    original_size: originalSize,
                    new_size: null,
                });
            });
    });
}

async function batchProcessSequential(files, outputDir, width, height, keepRatio, format, quality) {
    const start = Date.now();
    const results = [];

    for (const filePath of files) {
        const parsedPath = path.parse(filePath);
        const outputName = `${parsedPath.name}.${format}`;
        const outputPath = path.join(outputDir, outputName);

        const result = await processSingleImage(
            filePath,
            outputPath,
            width,
            height,
            keepRatio,
            format,
            quality
        );
        results.push(result);
    }

    const elapsed = Date.now() - start;
    const successCount = results.filter((r) => r.success).length;
    const failedCount = results.filter((r) => !r.success).length;

    return {
        total: files.length,
        success: successCount,
        failed: failedCount,
        total_time_ms: elapsed,
        results,
    };
}

async function batchProcessParallel(files, outputDir, width, height, keepRatio, format, quality) {
    const start = Date.now();

    const promises = files.map((filePath) => {
        const parsedPath = path.parse(filePath);
        const outputName = `${parsedPath.name}.${format}`;
        const outputPath = path.join(outputDir, outputName);

        return processSingleImage(
            filePath,
            outputPath,
            width,
            height,
            keepRatio,
            format,
            quality
        );
    });

    const results = await Promise.all(promises);

    const elapsed = Date.now() - start;
    const successCount = results.filter((r) => r.success).length;
    const failedCount = results.filter((r) => !r.success).length;

    return {
        total: files.length,
        success: successCount,
        failed: failedCount,
        total_time_ms: elapsed,
        results,
    };
}

async function main() {
    const args = process.argv.slice(2);
    const config = JSON.parse(args[0]);

    if (!fs.existsSync(config.output_dir)) {
        fs.mkdirSync(config.output_dir, { recursive: true });
    }

    let result;
    if (config.parallel) {
        result = await batchProcessParallel(
            config.files,
            config.output_dir,
            config.width,
            config.height,
            config.keep_ratio,
            config.format,
            config.quality
        );
    } else {
        result = await batchProcessSequential(
            config.files,
            config.output_dir,
            config.width,
            config.height,
            config.keep_ratio,
            config.format,
            config.quality
        );
    }

    console.log(JSON.stringify(result));
}

main().catch((err) => {
    console.error(JSON.stringify({ error: err.message }));
    process.exit(1);
});