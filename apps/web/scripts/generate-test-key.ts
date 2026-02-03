// scripts/generate-test-key.ts
// Run with: npx ts-node --project apps/web/tsconfig.json apps/web/scripts/generate-test-key.ts

import { LicenseManager } from '../src/lib/license';
import * as dotenv from 'dotenv';
import * as path from 'path';

dotenv.config({ path: path.resolve(__dirname, '../.env.local') });

async function main() {
    try {
        const lm = new LicenseManager();
        const key = await lm.generateLicenseKey('pro');
        console.log('--- GENERATED PRO LICENSE KEY ---');
        console.log(key);
        console.log('---------------------------------');
    } catch (e) {
        console.error('Error generating key. Make sure LICENSE_ENCRYPTION_KEY is set in .env.local');
        console.error(e);
    }
}

main();
