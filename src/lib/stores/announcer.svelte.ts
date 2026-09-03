/**
 * The app's status-message layer (audit 0.9.0, WCAG 4.1.3).
 *
 * Surfaces report outcomes visually — "Written ✓", a caution-hue scan
 * failure, a receipt — but a message that APPEARS after the page loaded is
 * invisible to a screen reader unless it lands in a live region. Rather
 * than make every footer a live region (each would then fight over the
 * user's attention, and footers contain layout, not messages), the shell
 * owns ONE visually-hidden polite region and anything with news `say`s
 * into it. The visual layer and this layer deliberately carry the SAME
 * sentences: what you see is what is announced.
 */
class Announcer {
  message = $state("");

  /** Announce a status. Identical consecutive messages are re-announced
   *  (a second "Written ✓" overwriting the first would be no change, and
   *  no change is no announcement), so equal text gets a growing space
   *  tail — still one visible sentence, always a new node value. */
  say(msg: string) {
    this.message =
      msg === this.message ? this.message + " " : msg;
  }
}

export const announcer = new Announcer();
