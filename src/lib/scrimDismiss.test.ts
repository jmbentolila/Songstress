import { describe, expect, it, vi } from "vitest";
import { scrimDismiss } from "./scrimDismiss";

/** Minimal element stub: the action only needs listener registration plus the
 *  `target` comparisons the handler makes. No jsdom in this project. */
function stubNode() {
  const listeners = new Map<string, (e: unknown) => void>();
  return {
    listeners,
    node: {
      addEventListener: (t: string, fn: (e: unknown) => void) => {
        listeners.set(t, fn);
      },
      removeEventListener: (t: string) => {
        listeners.delete(t);
      },
    } as unknown as HTMLElement,
  };
}

function press(
  target: unknown,
  opts: { button?: number; isPrimary?: boolean } = {},
) {
  return {
    button: opts.button ?? 0,
    isPrimary: opts.isPrimary ?? true,
    target,
  };
}

describe("scrimDismiss", () => {
  it("dismisses on a press that starts on the scrim itself", () => {
    const { listeners, node } = stubNode();
    const close = vi.fn();
    scrimDismiss(node, close);
    listeners.get("pointerdown")!(press(node));
    expect(close).toHaveBeenCalledTimes(1);
  });

  it("ignores a press that starts inside the panel (the text-selection case)", () => {
    const { listeners, node } = stubNode();
    const close = vi.fn();
    const panel = { tagName: "SECTION" }; // any descendant
    scrimDismiss(node, close);
    listeners.get("pointerdown")!(press(panel));
    expect(close).not.toHaveBeenCalled();
  });

  it("ignores right/middle presses and non-primary pointers", () => {
    const { listeners, node } = stubNode();
    const close = vi.fn();
    scrimDismiss(node, close);
    listeners.get("pointerdown")!(press(node, { button: 2 }));
    listeners.get("pointerdown")!(press(node, { button: 1 }));
    listeners.get("pointerdown")!(press(node, { isPrimary: false }));
    expect(close).not.toHaveBeenCalled();
  });

  it("no longer listens for click, which is what closed on a drag-release", () => {
    const { listeners, node } = stubNode();
    scrimDismiss(node, vi.fn());
    expect(listeners.has("click")).toBe(false);
    expect(listeners.has("pointerdown")).toBe(true);
  });

  it("follows an updated callback and cleans up on destroy", () => {
    const { listeners, node } = stubNode();
    const first = vi.fn();
    const second = vi.fn();
    const action = scrimDismiss(node, first);
    action.update(second);
    listeners.get("pointerdown")!(press(node));
    expect(first).not.toHaveBeenCalled();
    expect(second).toHaveBeenCalledTimes(1);
    action.destroy();
    expect(listeners.has("pointerdown")).toBe(false);
  });
});