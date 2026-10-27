import type { AccountHistory, Heatmap, HistoryView, WindowChart } from "./meter";

/**
 * The picture of one account's history to share (D-115), drawn from the History view's own
 * numbers: the account's label, the period and its dates, each window's chart with its 90% line,
 * its latest and highest share, the heatmap of when it was used, and where it came from. It is
 * the account's card as the window draws one (D-132), on the material (D-137): the card's fill,
 * corners and hairline edge, scaled up, on the colour of the macOS popover material, which is the
 * picture's own choice and not whatever the page happens to paint; and its charts have no plate,
 * as History's have none. It is drawn at 1200 by 675 CSS pixels, twice over, as a 2400 by 1350
 * PNG: the 16:9 shape social feeds show whole, with few words and large numbers so it reads when
 * shown small. Its text, card and chart colours and its font are the window's own, read from the
 * stylesheet, so it follows the window's theme. It is read away from the app, with nothing around
 * it to explain it, so a window's highest share is said only when it differs from its latest, and
 * the dashed line says it is at 90%.
 *
 * Privacy: the picture draws only what `shareStrings` returns, and that holds the account's
 * `label` (its provider and the name given it, or its plan), never its `name`, which can be its
 * address, nor a sign-in's place, a path or a token.
 */

/** The picture's width in CSS pixels. */
export const WIDTH = 1200;
/** The picture's height in CSS pixels. */
export const HEIGHT = 675;
/** How many pixels it has for each CSS pixel: a 2400 by 1350 PNG. */
export const SCALE = 2;
/** The request header naming the account a picture is of (`share::ACCOUNT_HEADER` in the app). */
export const ACCOUNT_HEADER = "quotadial-account";

/**
 * The picture's ground, the picture's own choice (D-137): the colour of the macOS popover
 * material in each theme, `#ececec` in light and `#2c2c2e` in dark, the same two the contrast
 * check takes for the material. It is not read from the page's `--background`, which is only what
 * a page with no system material behind it paints, so a change to the page's grey cannot change
 * the picture; check-contrast.mjs holds the picture's text and chart line to their ratios on these.
 */
export const MATERIAL = { light: "#ececec", dark: "#2c2c2e" } as const;

/** The two themes the window has, which are the picture's too. */
export type Theme = keyof typeof MATERIAL;

/** Where each colour comes from in the stylesheet. Text uses only text, secondary, muted, warning
 * and high, which check-contrast.mjs holds to 4.5:1 on the picture's card in both themes. */
const PROPERTIES = {
  text: "--text",
  /** An account's second line in the window; here the period and its dates. */
  secondary: "--secondary",
  muted: "--muted",
  warning: "--warning",
  high: "--high",
  /** The card's fill, white over the material: 70% in light, 6% in dark. */
  card: "--card",
  /** The card's hairline edge: black at 7% in light, clear in dark. */
  edge: "--card-edge",
  /** A chart's hairline at 0%, and its faint line at 100%. */
  base: "--chart-base",
  top: "--chart-top",
  /** A chart's line and the heatmap's squares. */
  line: "--bar-normal",
  /** The dashed line at 90%. */
  limit: "--bar-warning",
} as const;

/** The colours and font the picture takes from the window's stylesheet, and the theme it is in. */
export type Look = Record<keyof typeof PROPERTIES, string> & {
  /** The window's own font stack. */
  font: string;
  /** The theme the window shows, which picks the picture's material. */
  theme: Theme;
};

/** The look of the window's root element now, in the theme it shows. */
export function lookOf(root: Element): Look {
  const style = getComputedStyle(root);
  // The stylesheet's own test for its dark colours, so the picture cannot disagree with it.
  const dark = window.matchMedia("(prefers-color-scheme: dark)").matches;
  const look = { font: style.fontFamily, theme: dark ? "dark" : "light" } as Look;
  const entries = Object.entries(PROPERTIES) as [keyof typeof PROPERTIES, string][];
  for (const [name, property] of entries) {
    look[name] = style.getPropertyValue(property).trim();
  }
  return look;
}

/** A window past these shares is drawn in the warning and high colours, as the popover's meter. */
const WARNING_PCT = 70;
const HIGH_PCT = 90;
/** Where each chart's dashed line is drawn, as History draws it, and the words beside it. */
const LIMIT_PCT = 90;
const LIMIT = `${LIMIT_PCT}%`;

/** A place on the picture, in CSS pixels. */
export interface Box {
  x: number;
  y: number;
  width: number;
  height: number;
}

/**
 * The account's card, to scale with the window's. In the 360 px window a card is 336 px wide, 12 px
 * in from each side, with 10 px corners and 12 px of padding; here it is 1120 px wide, 40 px in
 * from each side of the 1200 px picture, so a pixel of the window is 10/3 pixels of the picture
 * and the corners and the padding are those, scaled. Its hairline edge is the window's 0.5 px,
 * drawn as 1.5 px so that it still shows when the picture is small.
 */
export const CARD = {
  x: 40,
  y: 40,
  width: 1120,
  height: 595,
  /** The window's 10 px corners. */
  radius: 33,
  /** The window's 12 px of padding. */
  padding: 40,
  /** The window's 0.5 px hairline edge. */
  edge: 1.5,
} as const;

/** Where things go, in CSS pixels: the card's contents, and where each line of them is written. */
const LEFT = CARD.x + CARD.padding;
const RIGHT = CARD.x + CARD.width - CARD.padding;
const ROOM = RIGHT - LEFT;
/** The heading: the card's title type, 14 px semibold times 10/3, with the period under it. */
const TITLE_SIZE = 47;
const TITLE_AT = 118;
const PERIOD_AT = 154;
/** Between the windows' columns. */
const GUTTER = 36;
/** The largest a window's latest share is drawn. */
const NUMBER = 84;
/** Each window's name, its latest share and its highest, by the baseline they are written on. */
const LABEL_AT = 204;
const LATEST_AT = 282;
const HIGHEST_AT = 311;
/** Where each chart begins, how tall it is, and the room kept at its right end for "90%". */
const CHART_TOP = 326;
const CHART_HEIGHT = 58;
const LIMIT_ROOM = 48;
/** The thickness of a chart's scale, its dashed line and its line. */
const RULE = 2;
const LINE = 3;
/**
 * The heatmap's words, its room for a day's label, where its rows begin, where its hours are
 * written, and the room between its last row and them. A week's rows are 18.9 px apart, room for
 * a day's name at 16 px to stand clear of the one above it.
 */
const CAPTION_AT = 430;
const DAY_ROOM = 92;
const HEAT_TOP = 445;
const HOURS_AT = 598;
const HOURS_ROOM = 24;
/** How tall the heatmap's 16 px words stand above their baseline, to set them level with a row. */
const WORD_RISE = 11.5;

/** The room of `count` windows' columns, and where each begins. */
function columnsOf(count: number): { width: number; at: (index: number) => number } {
  const columns = Math.max(1, count);
  const width = (ROOM - GUTTER * (columns - 1)) / columns;
  return { width, at: (index) => LEFT + index * (width + GUTTER) };
}

/**
 * Where each of `count` windows' charts is drawn, left to right: its lines run along the box's top
 * and bottom, and the room after it holds the dashed line's label.
 */
export function chartBoxes(count: number): Box[] {
  const { width, at } = columnsOf(count);
  return Array.from({ length: count }, (_, index) => ({
    x: at(index),
    y: CHART_TOP,
    width: width - LIMIT_ROOM,
    height: CHART_HEIGHT,
  }));
}

/** What one window says. Its level picks the colour of its latest share. */
interface WindowWords {
  label: string;
  latest: string;
  /** Its highest share, when it differs from its latest; otherwise nothing, its room left empty. */
  highest: string | null;
  level: "normal" | "warning" | "high";
  chart: WindowChart;
}

/** Everything the picture says, in the order it is drawn. */
interface Words {
  brand: string;
  site: string;
  title: string;
  period: string;
  windows: WindowWords[];
  heatmap: string;
  /** The rows of the heatmap that are labelled, with their day. */
  days: { row: number; text: string }[];
  hours: string[];
}

function words(view: HistoryView, account: AccountHistory): Words {
  const heatmap = account.heatmap;
  const used = heatmap.most > 0;
  const counted = heatmap.window.toLowerCase();
  const step = heatmap.days.length <= 7 ? 1 : 7;
  return {
    brand: "QuotaDial",
    site: "quotadial.com",
    title: account.label,
    period: [`The last ${view.days} days`, view.dates].filter(Boolean).join(", "),
    windows: account.windows.map((chart) => ({
      label: chart.label,
      latest: `${Math.floor(chart.latest)}%`,
      // Said only when it adds something, as for a session that reset during the period.
      highest:
        Math.floor(chart.highest) === Math.floor(chart.latest)
          ? null
          : `highest ${Math.floor(chart.highest)}%`,
      level:
        chart.latest >= HIGH_PCT ? "high" : chart.latest >= WARNING_PCT ? "warning" : "normal",
      chart,
    })),
    heatmap: used
      ? `When it was used, by hour${counted ? ` (${counted})` : ""}`
      : `No use in ${counted || "its"} readings in this period.`,
    // Every day of a week is named; over a month, every seventh, so the names keep their room.
    days: used ? heatmap.days.flatMap((text, row) => (row % step === 0 ? [{ row, text }] : [])) : [],
    hours: used ? ["0", "6", "12", "18"] : [],
  };
}

/**
 * Every string the picture of `account` draws, in the order drawn. Each is drawn whole, or cut
 * short to fit and ended with "…"; nothing else is written on the picture. It holds the account's
 * label and never its address, which check-window.mjs proves against what is drawn.
 */
export function shareStrings(view: HistoryView, account: AccountHistory): string[] {
  const said = words(view, account);
  return [
    said.brand,
    said.site,
    said.title,
    said.period,
    ...said.windows.flatMap((column) => [
      column.label,
      column.latest,
      ...(column.highest === null ? [] : [column.highest]),
      LIMIT,
    ]),
    said.heatmap,
    ...said.days.map((day) => day.text),
    ...said.hours,
  ];
}

/** Draws the picture of `account` on `context`, a canvas of WIDTH and HEIGHT times SCALE. */
export function drawShare(
  context: CanvasRenderingContext2D,
  view: HistoryView,
  account: AccountHistory,
  look: Look,
): void {
  const said = words(view, account);
  const pen = new Pen(context, look);
  context.save();
  context.setTransform(SCALE, 0, 0, SCALE, 0, 0);
  // The ground is the material, the picture's own choice (D-137); the card sits on it.
  context.fillStyle = MATERIAL[look.theme];
  context.fillRect(0, 0, WIDTH, HEIGHT);
  drawCard(context, look);

  // Where it came from, small, at the card's top right, on the baselines of its title and period.
  pen.use(600, 24, look.text);
  const brand = pen.write(said.brand, RIGHT, TITLE_AT, "right");
  pen.use(400, 20, look.muted);
  const site = pen.write(said.site, RIGHT, PERIOD_AT, "right");
  const beside = ROOM - Math.max(brand, site) - 48;
  // The account's heading as the window sets it: its label in the card's title type, and under it
  // a second line, here the period, in the colour of the window's address line.
  pen.use(600, TITLE_SIZE, look.text);
  pen.write(said.title, LEFT, TITLE_AT, "left", beside);
  pen.use(400, 25, look.secondary);
  pen.write(said.period, LEFT, PERIOD_AT, "left", beside);

  // Each window: its name, its latest share, large, its highest when that differs (its room is
  // kept either way, so the charts in a row stay level), and its chart with the dashed line's
  // label in small type at the line's right end, on the card.
  const { width, at } = columnsOf(said.windows.length);
  const charts = chartBoxes(said.windows.length);
  pen.use(600, NUMBER, look.text);
  const number = Math.min(NUMBER, Math.floor((NUMBER * width) / pen.measure("100%")));
  const limitAt = CHART_TOP + ((100 - LIMIT_PCT) / 100) * CHART_HEIGHT;
  said.windows.forEach((column, index) => {
    const x = at(index);
    pen.use(600, 25, look.text);
    pen.write(column.label, x, LABEL_AT, "left", width);
    pen.use(600, number, column.level === "normal" ? look.text : look[column.level]);
    pen.write(column.latest, x, LATEST_AT, "left", width);
    if (column.highest !== null) {
      pen.use(400, 21, look.muted);
      pen.write(column.highest, x, HIGHEST_AT, "left", width);
    }
    pen.use(400, 16, look.muted);
    pen.write(LIMIT, x + width, limitAt + 6, "right");
    drawChart(context, column.chart, charts[index], look);
  });

  // When it was used, by day and hour.
  pen.use(400, 21, look.muted);
  pen.write(said.heatmap, LEFT, CAPTION_AT, "left", ROOM);
  if (said.hours.length > 0) {
    drawHeatmap(context, pen, account.heatmap, said, look);
  }
  context.restore();
}

/**
 * The account's card on the ground, as the window draws one: its fill laid over the material, its
 * corners rounded, and its hairline edge just inside the outline, as the window's inset shadow is.
 * The edge is clear in dark, as the window's is.
 */
function drawCard(context: CanvasRenderingContext2D, look: Look): void {
  context.save();
  rounded(context, CARD.x, CARD.y, CARD.width, CARD.height, CARD.radius);
  context.fillStyle = look.card;
  context.fill();
  const half = CARD.edge / 2;
  rounded(
    context,
    CARD.x + half,
    CARD.y + half,
    CARD.width - CARD.edge,
    CARD.height - CARD.edge,
    CARD.radius - half,
  );
  context.lineWidth = CARD.edge;
  context.strokeStyle = look.edge;
  context.stroke();
  context.restore();
}

/** A straight line of `colour` from `from` to `to` along `y`, as the current line style draws it. */
function rule(
  context: CanvasRenderingContext2D,
  from: number,
  to: number,
  y: number,
  colour: string,
): void {
  context.beginPath();
  context.moveTo(from, y);
  context.lineTo(to, y);
  context.strokeStyle = colour;
  context.stroke();
}

/**
 * A window's share used over the period, on the card with no plate under it, as History draws it
 * (D-132): a faint line at 100% and a hairline at 0% give the scale, whole at the chart's edges,
 * and a dashed line marks 90%.
 */
function drawChart(
  context: CanvasRenderingContext2D,
  chart: WindowChart,
  box: Box,
  look: Look,
): void {
  const { x, y, width, height } = box;
  // Nothing is clipped to a plate now, so a reading outside the chart's range is held to its edge.
  const at = ([along, used]: [number, number]): [number, number] => [
    x + Math.min(1, Math.max(0, along)) * width,
    y + ((100 - Math.min(100, Math.max(0, used))) / 100) * height,
  ];
  context.save();
  context.lineWidth = RULE;
  rule(context, x, x + width, y, look.top);
  rule(context, x, x + width, y + height, look.base);
  context.setLineDash([6, 6]);
  rule(context, x, x + width, y + ((100 - LIMIT_PCT) / 100) * height, look.limit);
  context.setLineDash([]);
  const points = chart.points.map(at);
  if (points.length > 0) {
    // The share used is filled in beneath its line, so the shape reads when the picture is small.
    context.beginPath();
    context.moveTo(points[0][0], y + height);
    for (const [px, py] of points) {
      context.lineTo(px, py);
    }
    context.lineTo(points[points.length - 1][0], y + height);
    context.closePath();
    context.globalAlpha = 0.16;
    context.fillStyle = look.line;
    context.fill();
    context.globalAlpha = 1;
    context.beginPath();
    points.forEach(([px, py], index) =>
      index === 0 ? context.moveTo(px, py) : context.lineTo(px, py),
    );
    if (points.length === 1) {
      context.arc(points[0][0], points[0][1], 3, 0, 2 * Math.PI);
    }
    context.strokeStyle = look.line;
    context.lineWidth = LINE;
    context.lineJoin = "round";
    context.lineCap = "round";
    context.stroke();
  }
  context.restore();
}

/** The heatmap: a row a day, a square an hour, stronger for more use, as History draws it. */
function drawHeatmap(
  context: CanvasRenderingContext2D,
  pen: Pen,
  heatmap: Heatmap,
  said: Words,
  look: Look,
): void {
  const rows = heatmap.cells.length;
  const gap = rows <= 7 ? 3 : 1;
  const left = LEFT + DAY_ROOM;
  const cell = (RIGHT - left - 23 * gap) / 24;
  const bottom = HOURS_AT - HOURS_ROOM;
  const pitch = (bottom - HEAT_TOP + gap) / Math.max(1, rows);
  const height = pitch - gap;
  context.save();
  context.fillStyle = look.line;
  heatmap.cells.forEach((hours, row) => {
    hours.forEach((used, hour) => {
      context.globalAlpha = used > 0 ? 0.15 + (0.85 * used) / heatmap.most : 0.06;
      const x = left + hour * (cell + gap);
      rounded(context, x, HEAT_TOP + row * pitch, cell, height, Math.min(3, height / 2));
      context.fill();
    });
  });
  context.restore();
  pen.use(400, 16, look.muted);
  for (const day of said.days) {
    // Each name sits level with the middle of its row, and inside the grid: a month's rows are
    // thinner than a name, and the first and last would otherwise stand out of it.
    const middle = HEAT_TOP + day.row * pitch + height / 2;
    const at = Math.min(bottom, Math.max(HEAT_TOP + WORD_RISE, middle + WORD_RISE / 2));
    pen.write(day.text, left - 14, at, "right");
  }
  said.hours.forEach((hour, index) => {
    pen.write(hour, left + index * 6 * (cell + gap), HOURS_AT, "left");
  });
}

/** A rectangle with rounded corners, as a path. */
function rounded(
  context: CanvasRenderingContext2D,
  x: number,
  y: number,
  width: number,
  height: number,
  radius: number,
): void {
  const r = Math.max(0, Math.min(radius, width / 2, height / 2));
  context.beginPath();
  context.moveTo(x + r, y);
  context.arcTo(x + width, y, x + width, y + height, r);
  context.arcTo(x + width, y + height, x, y + height, r);
  context.arcTo(x, y + height, x, y, r);
  context.arcTo(x, y, x + width, y, r);
  context.closePath();
}

/**
 * Writes text in the window's font with tabular figures: each figure takes the width of the
 * widest, as `font-variant-numeric: tabular-nums` makes them in the window, which a canvas cannot
 * be told. Text too wide for its room is cut short with "…".
 */
class Pen {
  private readonly context: CanvasRenderingContext2D;
  private readonly look: Look;
  private figure = 0;

  constructor(context: CanvasRenderingContext2D, look: Look) {
    this.context = context;
    this.look = look;
  }

  /** Writes in `weight` at `size` CSS pixels, in `colour`, from here on. */
  use(weight: number, size: number, colour: string): void {
    this.context.font = `${weight} ${size}px ${this.look.font}`;
    this.context.fillStyle = colour;
    this.context.textAlign = "left";
    this.context.textBaseline = "alphabetic";
    this.figure = Math.max(...[..."0123456789"].map((digit) => this.context.measureText(digit).width));
  }

  /** How wide `text` is, its figures tabular. */
  measure(text: string): number {
    return runs(text).reduce(
      (total, run) =>
        total + (isFigures(run) ? run.length * this.figure : this.context.measureText(run).width),
      0,
    );
  }

  /** Writes `text` from `x` (or to it, aligned right) on the baseline `y`, within `room`, and
   * answers how wide it was. */
  write(
    text: string,
    x: number,
    y: number,
    align: "left" | "right",
    room = Number.POSITIVE_INFINITY,
  ): number {
    const shown = this.fit(text, room);
    const width = this.measure(shown);
    let at = align === "right" ? x - width : x;
    for (const run of runs(shown)) {
      if (isFigures(run)) {
        // Each figure is centred in the room of the widest.
        for (const figure of run) {
          const inset = (this.figure - this.context.measureText(figure).width) / 2;
          this.context.fillText(figure, at + inset, y);
          at += this.figure;
        }
      } else {
        this.context.fillText(run, at, y);
        at += this.context.measureText(run).width;
      }
    }
    return width;
  }

  /** `text`, or as much of it as fits in `room` with "…" after it. */
  private fit(text: string, room: number): string {
    if (this.measure(text) <= room) {
      return text;
    }
    const characters = Array.from(text);
    while (characters.length > 0 && this.measure(`${characters.join("").trimEnd()}…`) > room) {
      characters.pop();
    }
    return `${characters.join("").trimEnd()}…`;
  }
}

/** `text` in runs of figures and of everything else. */
function runs(text: string): string[] {
  return text.match(/[0-9]+|[^0-9]+/g) ?? [];
}

/** Whether a run from `runs` is of figures. */
function isFigures(run: string): boolean {
  return /^[0-9]/.test(run);
}

/** What to say when the picture cannot be drawn: one sentence that names what to do. */
const NOT_DRAWN =
  "QuotaDial could not draw the picture; close History, open it again and press Share image.";

/** The picture of `account`, drawn with `look`, as a PNG's bytes. */
export function shareImage(
  view: HistoryView,
  account: AccountHistory,
  look: Look,
): Promise<Uint8Array> {
  const canvas = document.createElement("canvas");
  canvas.width = WIDTH * SCALE;
  canvas.height = HEIGHT * SCALE;
  const context = canvas.getContext("2d");
  if (!context) {
    return Promise.reject(NOT_DRAWN);
  }
  drawShare(context, view, account, look);
  return new Promise((done, fail) => {
    canvas.toBlob((blob) => {
      if (!blob) {
        fail(NOT_DRAWN);
        return;
      }
      blob.arrayBuffer().then(
        (buffer) => done(new Uint8Array(buffer)),
        () => fail(NOT_DRAWN),
      );
    }, "image/png");
  });
}
