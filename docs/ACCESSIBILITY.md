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
- Enter or Space presses a button, Space ticks or clears a box, and Enter opens or closes "What
  QuotaDial does with this sign-in" and the other disclosures.
- History, Settings and Help open below the numbers and take the focus, so Tab carries on inside
  them. Done, at the foot of the window where those three buttons were, gives the focus back to the
  button that opened them.
- With many accounts the window scrolls, and its toolbar stays at its foot: History, Settings and
  Help (Done, under a panel) and Quit QuotaDial, with the line saying where the numbers come from.
  What scrolls is the number's line with the accounts or the panel under it, above the toolbar, and
  a faint line over the toolbar shows while there is more below. With the focus in them, the arrow
  keys, Page Up and Page Down, Home and End scroll them, and so does Space where the focus is on
  nothing Space presses, such as a panel as it opens; and with nothing focused, as in a window just
  opened, the first of those keys scrolls them. Tab reaches what they hold in the order shown
  and the toolbar after it, and scrolls each control into view; each view opens at its top. If they
  hold nothing Tab reaches, they are a Tab stop of their own, named for a screen reader, so that
  the keys can scroll them, and they become one as soon as the last control in them goes.
- A command QuotaDial asks you to run, such as the one that signs a Claude Code folder in again,
  is shown as code, and one click selects all of it, ready to copy. An account's email address can
  be selected and copied too.
- Escape hides the window.

## With a screen reader

- Every control says its name, what it is and its state: a box ticked or not, a disclosure open or
  closed, the period History shows.
- Each account is a section named by its heading: the name you gave it, or its provider, then its
  email address, so two accounts of one provider are told apart. The line under the number names
  the account it belongs to, and so does the tooltip of QuotaDial's icon.
- Each limit's bar is read with its words, such as "Weekly, 91% used, resets in 2 d 4 h".
- Where and when an account's limits were read is said once, after its last limit and before "What
  QuotaDial does with this sign-in", such as "From the API 3 min ago", when all of them were read from
  the same source at the same moment, as the limits of one answer are. When they were not, each
  limit says its own, after its reset.
- Each chart in History is read as what it shows, such as "Session: latest 18%, highest 18% in the
  last 7 days", and when an account was used is read as a line a day, such as "Mon 20: used 9:00
  to 12:00, most at 10:00 (12%)", in place of the grid of squares.
- An update's progress is read a step at a time, and a problem in Settings, History or Help is
  read as soon as it appears.

## Text size, colour and motion

- **Larger text:** Command and = (Control and = on Windows and Linux) zooms the window in, Command
  and - zooms it out, and Command and 0 sets it back. The window grows with its text to its tallest
  and then scrolls; nothing is cut off at 200%, and a long account name, address, folder, file name or
  command is broken where it must be to fit. A share used, such as 64%, is never split over two lines: the
  name beside it wraps instead. A short word, such as "Weekly" in History's captions, is never
  broken either: a caption wraps between its words. The
  toolbar stays at the window's foot while the accounts scroll, at 200% too, where it and the line
  under it take about a third of the window. The window zooms on, to 500% and more, and past about
  210% they would take most of it, so there the whole window scrolls as one, with the toolbar at
  the end.
- **Contrast:** text has at least 4.5:1 against what is behind it, an account's card, the
  capsules on it and a field's hint included, and the focus ring, the edges of text fields and
  History's lines at least 3:1, in both themes (WCAG 2.2 AA). The picture to share holds its
  charts' lines at 3:1 on its card too, in both themes, and the dashed 90% line on the soft fill
  under a chart's line as well.
- **Colour is never the only sign:** each bar has its percentage beside it in text, and the period
  History shows is bold as well as raised.
- **Less motion:** with Reduce motion on (macOS, Windows or your desktop's setting), nothing in the
  window moves, and scrolling is never animated.
- **Steady numbers:** every figure is the same width, so a number that changes does not move what
  is beside it.

## How it is checked

- Every build checks the contrast of each pair of colours in both themes, from the stylesheet
  itself ([`check-contrast.mjs`](../apps/tray/scripts/check-contrast.mjs)).
- CI opens the built window in Chrome on macOS, Windows and Linux
  ([`check-window.mjs`](../apps/tray/scripts/check-window.mjs)), over the accounts, Settings,
  History and Help, in both themes, for QuotaDial Free and Pro and for a Pro copy with ten
  accounts, and over History for a Pro account with seven limits, with made-up accounts, one of them with a very long address and others with a long name, plan, limit, folder, command and host, none with a space to break at. It presses Tab through every control and checks the order and the
  focus ring, presses each control with the keyboard and checks that it did something, checks
  what a screen reader is told of each, that a panel takes the focus and Done gives it back, that
  no text is cut off at 100% and 200%, that no share used, such as 64%, is drawn on two lines, at
  those and on to 500%, whatever name is beside it, that no word of four letters or more in a
  caption, such as "Weekly", is broken across two lines at 100% and 200%, that each account is a card of its own, apart from the next,
  with its email address shown inside it and read as part of its heading, that the line under the
  number names the account, that where and when an account's limits were read is said once for its
  card, in a quiet line under them and above its sign-in, which stays above it when the sign-in
  opens, or under each limit when they differ, and is told to a screen reader, that each command is
  shown whole as code and selected by one click,
  that what the window holds is as wide when it scrolls as when it does not, with the cards the
  same distance from each side and in line with the toolbar's labels, where a scroll bar takes room,
  that the toolbar stays inside the window and is what a click at its middle reaches at every
  place along the scroll of ten accounts, with nothing drawn under it and a faint line over it
  only while there is more below, that the keys and the wheel scroll the accounts and Tab brings a
  control into view with its ring, that a screen reader reads the number's line, the accounts or
  the panel, the toolbar and where the numbers come from in that order, that the app is told how
  tall everything is so that the window can grow, and that in a window under 300 px, as past about
  210% zoom, the whole page scrolls as one,
  that nothing moves with less motion asked for, and that the page reports no error. Each of these
  checks was first shown to fail on a window broken on purpose.
- The same names, roles, states and focus were read from the macOS accessibility interface of the
  app itself, which is what VoiceOver uses.

## Not there yet

- There is no keyboard shortcut that opens QuotaDial from anywhere.
- The checks above are automatic; nobody has yet gone through QuotaDial end to end with VoiceOver,
  Narrator, NVDA or Orca. Reports from people who use them are very welcome.
