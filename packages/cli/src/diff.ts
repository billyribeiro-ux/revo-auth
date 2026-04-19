/**
 * Minimal line-based 3-way merge (diff3-style).
 *
 * Given base (original template), ours (user-modified) and theirs (new template),
 * produce either a merged string or a conflict report with `<<<<<<<`/`=======`/`>>>>>>>`
 * markers and a list of conflict ranges.
 */

export interface MergeResult {
	ok: boolean;
	text: string;
	conflicts: ConflictRegion[];
}

export interface ConflictRegion {
	oursStart: number;
	oursEnd: number;
	theirsStart: number;
	theirsEnd: number;
}

function splitLines(input: string): string[] {
	if (input === '') return [];
	const lines = input.split(/\r?\n/);
	// Preserve trailing newline semantics: if input ended with newline, split produces
	// a trailing empty string; keep it so join round-trips correctly.
	return lines;
}

function joinLines(lines: string[]): string {
	return lines.join('\n');
}

/** LCS matrix (DP). Returns a 2D array of lengths. */
function lcsMatrix(a: string[], b: string[]): number[][] {
	const m = a.length;
	const n = b.length;
	const dp: number[][] = Array.from({ length: m + 1 }, () =>
		new Array<number>(n + 1).fill(0),
	);
	for (let i = 1; i <= m; i++) {
		for (let j = 1; j <= n; j++) {
			const rowPrev = dp[i - 1];
			const rowCurr = dp[i];
			if (!rowPrev || !rowCurr) continue;
			if (a[i - 1] === b[j - 1]) {
				rowCurr[j] = (rowPrev[j - 1] ?? 0) + 1;
			} else {
				rowCurr[j] = Math.max(rowPrev[j] ?? 0, rowCurr[j - 1] ?? 0);
			}
		}
	}
	return dp;
}

interface Chunk {
	matched: boolean;
	aStart: number;
	aEnd: number; // exclusive
	bStart: number;
	bEnd: number; // exclusive
}

/**
 * Produce chunks alternating between matched (both sides identical) and
 * unmatched (differing) regions.
 */
function chunkDiff(a: string[], b: string[]): Chunk[] {
	const dp = lcsMatrix(a, b);
	const pairs: { i: number; j: number }[] = [];
	let i = a.length;
	let j = b.length;
	while (i > 0 && j > 0) {
		if (a[i - 1] === b[j - 1]) {
			pairs.push({ i: i - 1, j: j - 1 });
			i--;
			j--;
		} else {
			const up = dp[i - 1]?.[j] ?? 0;
			const left = dp[i]?.[j - 1] ?? 0;
			if (up >= left) i--;
			else j--;
		}
	}
	pairs.reverse();

	const chunks: Chunk[] = [];
	let ai = 0;
	let bi = 0;
	for (const pair of pairs) {
		if (ai < pair.i || bi < pair.j) {
			chunks.push({
				matched: false,
				aStart: ai,
				aEnd: pair.i,
				bStart: bi,
				bEnd: pair.j,
			});
		}
		chunks.push({
			matched: true,
			aStart: pair.i,
			aEnd: pair.i + 1,
			bStart: pair.j,
			bEnd: pair.j + 1,
		});
		ai = pair.i + 1;
		bi = pair.j + 1;
	}
	if (ai < a.length || bi < b.length) {
		chunks.push({
			matched: false,
			aStart: ai,
			aEnd: a.length,
			bStart: bi,
			bEnd: b.length,
		});
	}
	return chunks;
}

/**
 * 3-way merge. We compute base->ours and base->theirs diffs, then walk base
 * forward. For each base line we look at whether ours/theirs made a change.
 */
export function merge3(
	base: string,
	ours: string,
	theirs: string,
): MergeResult {
	const baseLines = splitLines(base);
	const oursLines = splitLines(ours);
	const theirsLines = splitLines(theirs);

	// If one side matches base exactly, return the other.
	if (base === ours) return { ok: true, text: theirs, conflicts: [] };
	if (base === theirs) return { ok: true, text: ours, conflicts: [] };
	if (ours === theirs) return { ok: true, text: ours, conflicts: [] };

	const oursChunks = chunkDiff(baseLines, oursLines);
	const theirsChunks = chunkDiff(baseLines, theirsLines);

	// Build per-base-line mapping of "kept/replaced" plus inserted blocks at
	// boundaries.
	interface Edit {
		replace: boolean;
		with: string[];
	}
	const buildEdits = (
		chunks: Chunk[],
		otherLines: string[],
	): Map<number, Edit[]> => {
		const edits = new Map<number, Edit[]>();
		for (const chunk of chunks) {
			if (chunk.matched) continue;
			const replacement = otherLines.slice(chunk.bStart, chunk.bEnd);
			const key = chunk.aStart;
			const arr = edits.get(key) ?? [];
			arr.push({ replace: chunk.aEnd > chunk.aStart, with: replacement });
			edits.set(key, arr);
		}
		return edits;
	};

	const oursEdits = buildEdits(oursChunks, oursLines);
	const theirsEdits = buildEdits(theirsChunks, theirsLines);

	// Compute skip ranges (how many base lines each edit replaces).
	const oursReplaceEnd = new Map<number, number>();
	const theirsReplaceEnd = new Map<number, number>();
	for (const chunk of oursChunks) {
		if (!chunk.matched && chunk.aEnd > chunk.aStart) {
			oursReplaceEnd.set(chunk.aStart, chunk.aEnd);
		}
	}
	for (const chunk of theirsChunks) {
		if (!chunk.matched && chunk.aEnd > chunk.aStart) {
			theirsReplaceEnd.set(chunk.aStart, chunk.aEnd);
		}
	}

	const out: string[] = [];
	const conflicts: ConflictRegion[] = [];
	let idx = 0;
	while (idx <= baseLines.length) {
		const oursEdit = oursEdits.get(idx);
		const theirsEdit = theirsEdits.get(idx);

		if (oursEdit && theirsEdit) {
			const oursWith = oursEdit.flatMap((e) => e.with);
			const theirsWith = theirsEdit.flatMap((e) => e.with);
			if (joinLines(oursWith) === joinLines(theirsWith)) {
				out.push(...oursWith);
			} else {
				const start = out.length;
				out.push(
					'<<<<<<< ours',
					...oursWith,
					'=======',
					...theirsWith,
					'>>>>>>> theirs',
				);
				conflicts.push({
					oursStart: start + 1,
					oursEnd: start + 1 + oursWith.length,
					theirsStart: start + 2 + oursWith.length,
					theirsEnd: start + 2 + oursWith.length + theirsWith.length,
				});
			}
			const skip = Math.max(
				oursReplaceEnd.get(idx) ?? idx,
				theirsReplaceEnd.get(idx) ?? idx,
			);
			idx = skip === idx ? idx : skip;
			if (
				idx < baseLines.length &&
				!(oursReplaceEnd.get(idx) || theirsReplaceEnd.get(idx))
			) {
				// no replacement at this base line; emit the base line and advance
				const baseLine = baseLines[idx];
				if (baseLine !== undefined) out.push(baseLine);
				idx++;
			} else if (idx === baseLines.length) {
				idx++;
			}
			continue;
		}

		if (oursEdit) {
			out.push(...oursEdit.flatMap((e) => e.with));
			const end = oursReplaceEnd.get(idx);
			if (end !== undefined) idx = end;
			continue;
		}
		if (theirsEdit) {
			out.push(...theirsEdit.flatMap((e) => e.with));
			const end = theirsReplaceEnd.get(idx);
			if (end !== undefined) idx = end;
			continue;
		}

		if (idx < baseLines.length) {
			const baseLine = baseLines[idx];
			if (baseLine !== undefined) out.push(baseLine);
		}
		idx++;
	}

	return { ok: conflicts.length === 0, text: joinLines(out), conflicts };
}
