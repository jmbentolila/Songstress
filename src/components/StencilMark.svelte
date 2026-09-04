<!-- The app's stencil mark, glyph-only: the microphone from
     assets/app-icon.svg without its tile. This is what the app shows
     wherever artwork is absent — grid tiles, the expanded panel, the
     playbar's square — so "no cover" reads as "this app's quiet room"
     instead of a dead rectangle (owner request 2026-09-04).
     Draws in currentColor; the container chooses size, tint and opacity.
     The mask id is document-global, and every instance of this component
     is byte-identical, so N copies resolving to the first one is exactly
     as good as one. -->
<svg class="stencil-mark" viewBox="0 0 512 512" aria-hidden="true" focusable="false">
  <defs>
    <mask id="songstress-stencil">
      <rect width="512" height="512" fill="black" />
      <g transform="translate(252 268) rotate(40) translate(-252 -268)">
        <rect x="192" y="222" width="120" height="180" rx="60" fill="white" />
        <path d="M 156 312 a 96 96 0 0 0 192 0" fill="none" stroke="white" stroke-width="30" stroke-linecap="round" />
        <rect x="226" y="404" width="52" height="72" rx="24" fill="white" />
        <rect x="182" y="258" width="140" height="18" fill="black" />
        <rect x="182" y="300" width="140" height="18" fill="black" />
        <path d="M 200 178 A 62 62 0 0 1 304 178" fill="none" stroke="white" stroke-width="26" stroke-linecap="round" />
        <path d="M 172 122 A 96 96 0 0 1 332 122" fill="none" stroke="white" stroke-width="22" stroke-linecap="round" opacity="0.72" />
      </g>
    </mask>
  </defs>
  <rect width="512" height="512" fill="currentColor" mask="url(#songstress-stencil)" />
</svg>

<style>
  svg {
    display: block;
    width: 100%;
    height: 100%;
  }
</style>
