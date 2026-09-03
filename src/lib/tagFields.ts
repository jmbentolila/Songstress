/**
 * Shared surface of the two tag editors (Tag Editor Redesign, Phase C split).
 *
 * The field model, the census types and the cheap validation — everything
 * the album modal and the track modal agree on, so the pair cannot drift.
 * The write-set rule lives on the Rust side; this file is what the form
 * shows before that rule runs.
 */

/** File-level tag surface mirrored from the Rust TrackTags (serde names). */
export interface TrackTags {
  title: string;
  artist: string;
  albumArtist: string;
  album: string;
  year: number | null;
  trackNo: number | null;
  trackTotal: number | null;
  discNo: number | null;
  discTotal: number | null;
  genre: string;
  composer: string;
  label: string;
  comment: string;
  grouping: string;
}

/** The track modal's identity row (Rust `track_file`). Paths travel out as
 *  DISPLAY strings only — revealing stays `reveal_container`'s job. */
export interface TrackFile {
  file: string;
  folder: string;
  path: string;
  staged: boolean;
  albumId: string;
}

export interface AlbumTagsApi extends Omit<TrackTags, "title" | "artist"> {
  albumArtistDisputed: boolean;
  albumDisputed: boolean;
  yearDisputed: boolean;
  genreDisputed: boolean;
  composerDisputed: boolean;
  labelDisputed: boolean;
  groupingDisputed: boolean;
  commentDisputed: boolean;
  trackTotalDisputed: boolean;
  discTotalDisputed: boolean;
  conflicts: FieldCensus[];
  fileCount: number;
}

/** One field's census: distinct non-empty values with counts (≥2 is a
 *  dispute), and how many files carry any value. The chips and the
 *  blast-radius line both read this. */
export interface FieldCensus {
  field: string;
  values: { value: string; count: number }[];
  present: number;
}

/** Everything editable lives in strings; "" ⇔ absent (empty clears keys). */
export interface Editable {
  title: string;
  artist: string;
  albumArtist: string;
  album: string;
  year: string;
  trackNo: string;
  trackTotal: string;
  discNo: string;
  discTotal: string;
  genre: string;
  composer: string;
  label: string;
  comment: string;
  grouping: string;
}

export type FieldKey = keyof Editable;

export interface FieldDef {
  key: FieldKey;
  label: string;
  /** The field is small by nature (year, the numbers): fixed input width. */
  half?: boolean;
}

export const FIELDS: FieldDef[] = [
  { key: "title", label: "Title" },
  { key: "artist", label: "Artist" },
  // Album before Album artist: the album is what people look for; the
  // album-artist field is the disambiguator beneath it (owner ruling).
  { key: "album", label: "Album" },
  { key: "albumArtist", label: "Album artist" },
  { key: "genre", label: "Genre" },
  { key: "year", label: "Year", half: true },
  // Numbering sits with the identity fields, not under Composer: fixing a
  // track/disc number is the everyday edit, writing a composer is not
  // (owner ruling).
  { key: "trackNo", label: "Track #", half: true },
  { key: "discNo", label: "Disc #", half: true },
  { key: "composer", label: "Composer" },
  { key: "label", label: "Label" },
  { key: "grouping", label: "Grouping" },
  { key: "comment", label: "Comment" },
];

export const numStr = (n: number | null): string => (n === null ? "" : String(n));
export const strNum = (s: string): number | null =>
  s.trim() === "" ? null : Number(s);

export function toEditable(t: Partial<TrackTags>): Editable {
  return {
    title: t.title ?? "",
    artist: t.artist ?? "",
    albumArtist: t.albumArtist ?? "",
    album: t.album ?? "",
    year: numStr(t.year ?? null),
    trackNo: numStr(t.trackNo ?? null),
    trackTotal: numStr(t.trackTotal ?? null),
    discNo: numStr(t.discNo ?? null),
    discTotal: numStr(t.discTotal ?? null),
    genre: t.genre ?? "",
    composer: t.composer ?? "",
    label: t.label ?? "",
    comment: t.comment ?? "",
    grouping: t.grouping ?? "",
  };
}

/** The album-level fields a save may carry; keys match TouchedFields. */
export const SHARED: FieldKey[] = [
  "albumArtist",
  "album",
  "year",
  "genre",
  "composer",
  "label",
  "comment",
  "grouping",
  "trackTotal",
  "discTotal",
];

/** Labels for the numeric fields — the error line names them the way the
 *  inputs do (two of them never render as their own row). */
export const NUM_LABELS: Partial<Record<FieldKey, string>> = {
  year: "Year",
  trackNo: "Track #",
  trackTotal: "Track total",
  discNo: "Disc #",
  discTotal: "Disc total",
};

/**
 * Cheap inline validation (Rust re-validates): junk used to save as a
 * silent null — a "clear" dressed as a value. Now it never reaches Save.
 * Keys, not labels: the grid styles by key.
 */
export function badFieldKeys(edit: Editable): Set<FieldKey> {
  const bad = new Set<FieldKey>();
  if (edit.year.trim() !== "" && !/^\d{1,4}$/.test(edit.year.trim())) bad.add("year");
  for (const k of ["trackNo", "trackTotal", "discNo", "discTotal"] as const)
    if (edit[k].trim() !== "" && !/^\d{1,6}$/.test(edit[k].trim())) bad.add(k);
  return bad;
}
