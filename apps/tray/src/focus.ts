import { useEffect, useRef } from "preact/hooks";

/**
 * A panel takes the focus when it opens, so Tab carries on inside it and a screen reader says
 * where it is (#52). The footer button that opened it is gone while it is open, so without this
 * the focus would fall back to the start of the window.
 */
export function usePanelFocus() {
  const panel = useRef<HTMLElement>(null);
  useEffect(() => {
    panel.current?.focus();
  }, []);
  return panel;
}
