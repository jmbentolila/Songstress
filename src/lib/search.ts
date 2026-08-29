/** Case- and diacritic-insensitive folding for search matching. */
export function fold(s: string): string {
  return s
    .toLowerCase()
    .normalize("NFD")
    .replace(/\p{M}/gu, "");
}

export interface SearchableAlbum {
  title: string;
  artistName: string;
  trackTitles: string[];
}

/** Album matches by its own title (artists are matched by the sidebar list
 *  itself — "hello" must not surface Helloween albums in the grid). */
export function albumTitleMatches(
  album: { title: string },
  query: string,
): boolean {
  const q = fold(query.trim());
  if (q === "") return true;
  return fold(album.title).includes(q);
}

/** Album matches because at least one of its tracks matches. */
export function albumTrackMatches(
  album: { trackTitles: string[] },
  query: string,
): boolean {
  const q = fold(query.trim());
  if (q === "") return true;
  return album.trackTitles.some((t) => fold(t).includes(q));
}

/** True when the album matches the query by its own title, its artist's
 *  name, or any track title. Empty query matches everything. */
export function albumMatches(album: SearchableAlbum, query: string): boolean {
  return albumTitleMatches(album, query) || albumTrackMatches(album, query);
}
