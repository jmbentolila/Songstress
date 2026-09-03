/**
 * The artwork instruction a tag-editor save carries (mirrors the Rust
 * `ArtChange`, externally tagged serde). Externally-tagged means the JSON is
 * exactly: "keep" | "clear" | {hash} | {upload:{image,mime}}.
 */
export type ArtChange = "keep" | "clear" | { hash: string } | { upload: { image: string; mime: string } };

export interface ArtCandidate {
  hash: string;
  label: string;
  /** Files this image is embedded in (0 = folder art only). */
  count: number;
  /** Filename when a folder-art copy exists. */
  folder: string | null;
  preview: string;
  /** 512px variant for the lightbox. */
  full: string;
}

export interface ArtInventory {
  candidates: ArtCandidate[];
  current: string | null;
  folderArt: string | null;
  filesWithArt: number;
  trackPics: string[];
}

/** Magic-byte sniffing client-side so the data: URL renders truthfully
 *  (the Rust save re-sniffs anyway; this only feeds the tile preview). */
export function sniffMime(bytes: Uint8Array): string {
  const b = bytes;
  const four = (o: number) => String.fromCharCode(b[o], b[o + 1], b[o + 2], b[o + 3]);
  if (b[0] === 0xff && b[1] === 0xd8) return "image/jpeg";
  if (b[0] === 0x89 && b[1] === 0x50 && b[2] === 0x4e && b[3] === 0x47) return "image/png";
  if (four(0) === "RIFF" && four(8) === "WEBP") return "image/webp";
  if (four(0) === "GIF8") return "image/gif";
  if (b[0] === 0x49 && b[1] === 0x49 && b[2] === 0x2a && b[3] === 0) return "image/tiff";
  if (b[0] === 0x4d && b[1] === 0x4d && b[2] === 0 && b[3] === 0x2a) return "image/tiff";
  return "";
}

export function toBase64(bytes: Uint8Array): string {
  let bin = "";
  const chunk = 0x8000;
  for (let i = 0; i < bytes.length; i += chunk) {
    bin += String.fromCharCode(...bytes.subarray(i, i + chunk));
  }
  return btoa(bin);
}
