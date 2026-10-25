# Using QuotaDial by keyboard and screen reader

QuotaDial's window works with the keyboard alone and with a screen reader, in the light and dark
themes and at any text size. This page says what to expect, what is checked, and what is not
there yet. If something here does not work for you, please say so: open an issue or write to
support@quotadial.com.

## Opening QuotaDial

- **macOS:** click the number in the menu bar. With VoiceOver, move to the menu bar
  (VO-M, then VO-M again for the menu extras), find QuotaDial and press it (VO-Space). With Full
  Keyboard Access on, Control-F8 moves to the menu extras: move to QuotaDial and press Space. Either
  way the popover opens, as a click opens it; until QuotaDial has been clicked once it opens in the
  middle of the screen rather than under the icon. Opening QuotaDial again while it runs, from
  Spotlight (Command-Space, then type QuotaDial) or Launchpad, opens the popover too.
- **Windows:** click the icon in the notification area. With the keyboard, Windows-B moves to the
  notification area; QuotaDial's menu there has Open QuotaDial.
- **Linux:** QuotaDial's menu in the panel has Open QuotaDial.

## In the window

- Tab and Shift-Tab move through every control in the order it is shown, and each draws a blue
  ring while it has the focus.
- Enter or Space presses a button, Space ticks or clears a box, and Enter opens or closes "How
  QuotaDial reads this sign-in" and the other disclosures.
- History, Settings and Help open below the numbers and take the focus, so Tab carries on inside
  them. Done gives the focus back to the button that opened them.
- Escape hides the window.

## With a screen reader

- Every control says its name, what it is and its state: a box ticked or not, a disclosure open or
  closed, the period History shows.
- Each limit's bar is read with its words, such as "Weekly, 91% used, resets in 2 d 4 h".
- Each chart in History is read as what it shows, such as "Session: latest 18%, highest 18% in the
  last 7 days", and when an account was used is read as a line a day, such as "Mon 20: used 9:00
  to 12:00, most at 10:00 (12%)", in place of the grid of squares.
- An update's progress is read a step at a time, and a problem in Settings, History or Help is
  read as soon as it appears.

## Text size, colour and motion

- **Larger text:** Command and = (Control and = on Windows and Linux) zooms the window in, Command
  and - zooms it out, and Command and 0 sets it back. The window grows with its text to its tallest
  and then scrolls; nothing is cut off at 200%, and a long account name or address wraps.
- **Contrast:** text has at least 4.5:1 against what is behind it, and the focus ring and the
  edges of text fields at least 3:1, in both themes (WCAG 2.2 AA).
- **Colour is never the only sign:** each bar has its percentage beside it in text, and the period
  History shows is bold as well as blue.
- **Less motion:** with Reduce motion on (macOS, Windows or your desktop's setting), nothing in the
  window moves.
- **Steady numbers:** every figure is the same width, so a number that changes does not move what
  is beside it.

## How it is checked

- Every build checks the contrast of each pair of colours in both themes, from the stylesheet
  itself ([`check-contrast.mjs`](../apps/tray/scripts/check-contrast.mjs)).
- CI opens the built window in Chrome on macOS, Windows and Linux
  ([`check-window.mjs`](../apps/tray/scripts/check-window.mjs)), over the accounts, Settings,
  History and Help, in both themes, for QuotaDial Free and Pro, with made-up accounts, one of them
  with a very long address. It presses Tab through every control and checks the order and the
  focus ring, presses each control with the keyboard and checks that it did something, checks
  what a screen reader is told of each, that a panel takes the focus and Done gives it back, that
  no text is cut off at 100% and 200%, that nothing moves with less motion asked for, and that the
  page reports no error. Each of these checks was first shown to fail on a window broken on
  purpose.
- The same names, roles, states and focus were read from the macOS accessibility interface of the
  app itself, which is what VoiceOver uses.

## Not there yet

- There is no keyboard shortcut that opens QuotaDial from anywhere.
- The checks above are automatic; nobody has yet gone through QuotaDial end to end with VoiceOver,
  Narrator, NVDA or Orca. Reports from people who use them are very welcome.
