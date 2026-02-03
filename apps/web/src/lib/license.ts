export class LicenseManager {
    private encryptionKey: Uint8Array;

    constructor() {
        const key = process.env.LICENSE_ENCRYPTION_KEY;
        if (!key || key.length !== 64) {
            throw new Error('LICENSE_ENCRYPTION_KEY must be a 64-char hex string (32 bytes)');
        }
        const bytes = key.match(/.{1,2}/g)!.map((byte) => parseInt(byte, 16));
        this.encryptionKey = new Uint8Array(bytes);
    }

    async generateLicenseKey(tier: 'free' | 'basic' | 'pro'): Promise<string> {
        const data = {
            tier,
            created: Date.now(),
            random: Array.from(crypto.getRandomValues(new Uint8Array(8)))
                .map((b) => b.toString(16).padStart(2, '0'))
                .join('')
        };

        const json = JSON.stringify(data);
        const encoder = new TextEncoder();
        const encodedData = encoder.encode(json);

        const iv = crypto.getRandomValues(new Uint8Array(12));
        const cryptoKey = await crypto.subtle.importKey(
            'raw',
            this.encryptionKey,
            { name: 'AES-GCM' },
            false,
            ['encrypt']
        );

        const encryptedContent = await crypto.subtle.encrypt(
            { name: 'AES-GCM', iv },
            cryptoKey,
            encodedData
        );

        const encryptedArray = new Uint8Array(encryptedContent);
        const packed = new Uint8Array(iv.length + encryptedArray.length);
        packed.set(iv);
        packed.set(encryptedArray, iv.length);

        const hex = Array.from(packed)
            .map((b) => b.toString(16).padStart(2, '0'))
            .join('')
            .toUpperCase();

        return `VT-${hex.slice(0, 4)}-${hex.slice(4, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}`;
    }

    async validateLicenseKey(key: string): Promise<{ valid: boolean; tier?: 'free' | 'basic' | 'pro' }> {
        try {
            const hex = key.replace('VT-', '').replace(/-/g, '');
            const bytes = hex.match(/.{1,2}/g)!.map((byte) => parseInt(byte, 16));
            const packed = new Uint8Array(bytes);

            const iv = packed.slice(0, 12);
            const encryptedData = packed.slice(12);

            const cryptoKey = await crypto.subtle.importKey(
                'raw',
                this.encryptionKey,
                { name: 'AES-GCM' },
                false,
                ['decrypt']
            );

            const decryptedBuffer = await crypto.subtle.decrypt(
                { name: 'AES-GCM', iv },
                cryptoKey,
                encryptedData
            );

            const decoder = new TextDecoder();
            const data = JSON.parse(decoder.decode(decryptedBuffer));

            return { valid: true, tier: data.tier };
        } catch (e) {
            console.error('License validation failed:', e);
            return { valid: false };
        }
    }
}
