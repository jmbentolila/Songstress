import type { Album, Artist, Track } from "./types";
import { compareByName } from "./sort";

function mulberry32(seed: number): () => number {
  return () => {
    seed |= 0;
    seed = (seed + 0x6d2b79f5) | 0;
    let t = Math.imul(seed ^ (seed >>> 15), 1 | seed);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const WORDS_A = [
  "Ashen", "Crimson", "Hollow", "Silent", "Obsidian", "Wandering",
  "Fractured", "Eternal", "Velvet", "Gilded", "Pale", "Iron",
];
const WORDS_B = [
  "Throne", "Choir", "Horizon", "Requiem", "Mirror", "Crown",
  "Lament", "Sanctum", "Vigil", "Echoes", "Embers", "Prophecy",
];

export const artists: Artist[] = [
  { id: "ancient-bards", name: "Ancient Bards", sortName: "ancient bards" },
  { id: "avantasia", name: "Avantasia", sortName: "avantasia" },
  { id: "ayreon", name: "Ayreon", sortName: "ayreon" },
  { id: "blind-guardian", name: "Blind Guardian", sortName: "blind guardian" },
  { id: "epica", name: "Epica", sortName: "epica" },
  { id: "ex-libris", name: "Ex Libris", sortName: "ex libris" },
  { id: "ghost", name: "Ghost", sortName: "ghost" },
  { id: "helloween", name: "Helloween", sortName: "helloween" },
  { id: "various-artists", name: "Various Artists", sortName: "various artists" },
].sort((a, b) => compareByName(a.name, b.name));

interface Seed {
  albumId: string;
  artistId: string;
  title: string;
  year: number;
  trackCount?: number;
  discCount?: number;
}

const seeds: Seed[] = [
  { albumId: "blind-guardian-nightfall", artistId: "blind-guardian", title: "Beneath the Silvered Sky", year: 1998 },
  { albumId: "blind-guardian-red-mirror", artistId: "blind-guardian", title: "The Red Mirror", year: 2015 },
  { albumId: "blind-guardian-god-machine", artistId: "blind-guardian", title: "God Machine", year: 2022 },
  { albumId: "epica-requiem", artistId: "epica", title: "Requiem for the Indifferent", year: 2012 },
  { albumId: "epica-aspiral", artistId: "epica", title: "Aspiral", year: 2025 },
  { albumId: "ghost-prequelle", artistId: "ghost", title: "Prequelle", year: 2018 },
  { albumId: "ghost-skeleta", artistId: "ghost", title: "Skeletá", year: 2022 },
  { albumId: "ayreon-human-equation", artistId: "ayreon", title: "The Human Equation", year: 2004 },
  { albumId: "avantasia-mystery-time", artistId: "avantasia", title: "The Mystery of Time", year: 2013 },
  { albumId: "helloween-giants-monsters", artistId: "helloween", title: "Giants & Monsters", year: 2021 },
  { albumId: "ancient-bards-artifex", artistId: "ancient-bards", title: "Artifex", year: 2019 },
  { albumId: "ex-libris-ann", artistId: "ex-libris", title: "Ann", year: 2017 },
  {
    albumId: "various-artists-anison-no-kokoro",
    artistId: "various-artists",
    title: "Anison no Kokoro",
    year: 2016,
    trackCount: 34,
  },
  {
    albumId: "helloween-keeper-live",
    artistId: "helloween",
    title: "Keepers of the Seven Keys — Live",
    year: 2019,
    trackCount: 22,
    discCount: 2,
  },
];

export const albums: Album[] = seeds
  .map((s) => ({
    id: s.albumId,
    artistId: s.artistId,
    title: s.title,
    year: s.year,
    cover: `/covers/${s.albumId}.jpg`,
  }))
  .sort((a, b) => {
    if (a.year !== b.year) return (a.year ?? Infinity) - (b.year ?? Infinity);
    return compareByName(a.title, b.title);
  });

const rand = mulberry32(0x5066);

export const tracksByAlbum: Map<string, Track[]> = new Map();
for (const seed of seeds) {
  const count = seed.trackCount ?? 8 + Math.floor(rand() * 6);
  const discCount = seed.discCount ?? 1;
  const perDisc = Math.ceil(count / discCount);
  const tracks: Track[] = [];
  for (let i = 0; i < count; i++) {
    const title =
      i === 0
        ? `${WORDS_A[Math.floor(rand() * WORDS_A.length)]} ${WORDS_B[Math.floor(rand() * WORDS_B.length)]}`
        : `${WORDS_B[Math.floor(rand() * WORDS_B.length)]} of the ${WORDS_A[Math.floor(rand() * WORDS_A.length)]} ${WORDS_B[Math.floor(rand() * WORDS_B.length)]}`;
    tracks.push({
      id: `${seed.albumId}/${i}`,
      albumId: seed.albumId,
      disc: Math.min(discCount, Math.floor(i / perDisc) + 1),
      track: (i % perDisc) + 1,
      title,
      durationSec: 180 + Math.floor(rand() * 240),
      // One guest spot so the playbar's track-artist line has something to
      // prefer in fake mode; everything else falls back to the album artist.
      ...(seed.artistId === "various-artists" && i === 1
        ? { artist: "Guest Vocalist" }
        : {}),
    });
  }
  tracksByAlbum.set(seed.albumId, tracks);
}

export function tracksOf(albumId: string): Track[] {
  return tracksByAlbum.get(albumId) ?? [];
}

export function artistOf(album: Album): Artist | undefined {
  return artists.find((a) => a.id === album.artistId);
}
