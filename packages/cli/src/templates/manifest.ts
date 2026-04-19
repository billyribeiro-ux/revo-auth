import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname } from 'pathe';
import { z } from 'zod';

export const MANIFEST_VERSION = 1;

const manifestFileSchema = z.object({
	templateHash: z.string(),
	writtenHash: z.string(),
	template: z.string().optional(),
});

const manifestSchema = z.object({
	version: z.number(),
	cliVersion: z.string(),
	files: z.record(manifestFileSchema),
});

export type ManifestFileEntry = z.infer<typeof manifestFileSchema>;
export type Manifest = z.infer<typeof manifestSchema>;

export function emptyManifest(cliVersion: string): Manifest {
	return { version: MANIFEST_VERSION, cliVersion, files: {} };
}

export function readManifest(path: string): Manifest | null {
	if (!existsSync(path)) return null;
	const raw = readFileSync(path, 'utf8');
	const parsed = manifestSchema.safeParse(JSON.parse(raw));
	if (!parsed.success) {
		throw new Error(
			`Invalid manifest at ${path}: ${parsed.error.issues.map((i) => i.message).join('; ')}`,
		);
	}
	return parsed.data;
}

export function writeManifest(path: string, manifest: Manifest): void {
	const dir = dirname(path);
	if (!existsSync(dir)) mkdirSync(dir, { recursive: true });
	writeFileSync(path, `${JSON.stringify(manifest, null, 2)}\n`, 'utf8');
}

export function setManifestEntry(
	manifest: Manifest,
	relPath: string,
	entry: ManifestFileEntry,
): void {
	manifest.files[relPath] = entry;
}
