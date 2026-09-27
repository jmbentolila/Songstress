import { describe, expect, it } from "vitest";
import { library } from "./library.svelte";
import { currentTrack, playback, reanchorCurrent } from "./playback.svelte";

/**
 * A retag can move the PLAYING row to another album: the file's path — and so
 * the track id — survives, while `tracks.album_id` does not. Before this was
 * fixed, `currentTrack()` resolved through the stale album id, returned null,
 * and the playbar read "Nothing playing" over audio mpv was still playing
 * (owner report, 2026-09-26). These two tests pin the resolution rule and the
 * re-anchor down without needing a DOM.
 */
describe("playback context survives a row changing album", () => {
  it("finds the playing track library-wide when the album id is stale", () => {
    const album = library.albums[0];
    const track = library.tracksOf(album.id)[0];
    expect(track, "the fake library needs a track to test with").toBeTruthy();

    playback.current = {
      albumId: "al-moved-away", // what a retag leaves behind
      trackIndex: 0,
      trackId: track.id,
    };
    expect(currentTrack()?.id).toBe(track.id);
    expect(currentTrack()?.albumId).toBe(album.id);
  });

  it("re-anchors the context at the row's new home", () => {
    const album = library.albums[1];
    const tracks = library.tracksOf(album.id);
    const track = tracks[tracks.length - 1];
    playback.current = { albumId: "al-moved-away", trackIndex: 7, trackId: track.id };

    reanchorCurrent();
    expect(playback.current?.albumId).toBe(album.id);
    expect(playback.current?.trackIndex).toBe(tracks.length - 1);
    expect(playback.current?.trackId).toBe(track.id);
  });

  it("leaves an unknown track id alone (nothing to re-anchor to)", () => {
    playback.current = { albumId: "al-x", trackIndex: 3, trackId: "tr-does-not-exist" };
    reanchorCurrent();
    expect(playback.current?.albumId).toBe("al-x");
  });

  it("falls back to the album index when the context has no track id", () => {
    const album = library.albums[0];
    const track = library.tracksOf(album.id)[0];
    playback.current = { albumId: album.id, trackIndex: 0 };
    expect(currentTrack()?.id).toBe(track.id);
  });
});