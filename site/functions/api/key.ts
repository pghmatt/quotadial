/**
 * `POST /api/key` on quotadial.com: the key page sends `{ "licence": "<receipt key>" }` and gets
 * `{ "key", "licence" }` or `{ "error" }` back (#54, D-102). Nothing is logged or kept, and the
 * answer is never cached.
 */

import { type Env, issue } from "../../lib/key.ts";

export async function onRequestPost(context: { request: Request; env: Env }): Promise<Response> {
  let orderKey = "";
  try {
    const sent: unknown = await context.request.json();
    if (typeof sent === "object" && sent !== null && typeof (sent as { licence?: unknown }).licence === "string") {
      orderKey = (sent as { licence: string }).licence;
    }
  } catch {
    // An unreadable request is answered as a key that is not one.
  }
  const answer = await issue(orderKey, context.env, (url, init) => fetch(url, init));
  return new Response(JSON.stringify(answer.body), {
    status: answer.status,
    headers: {
      "Content-Type": "application/json",
      "Cache-Control": "no-store",
      "X-Content-Type-Options": "nosniff",
    },
  });
}
