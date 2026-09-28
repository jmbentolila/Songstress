/** Live grid geometry shared by the album grid and the Appearance pane.
 * Session-only and never persisted: the tile-size slider needs the measured
 * grid width to offer only the stops that actually change the layout. */
export const layout = $state({
  gridWidth: 0,
});
