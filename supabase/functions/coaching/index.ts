import { createClient } from "https://esm.sh/@supabase/supabase-js@2";

const ANTHROPIC_API_KEY = Deno.env.get("ANTHROPIC_API_KEY")!;
const SUPABASE_URL = Deno.env.get("SUPABASE_URL")!;
const SUPABASE_ANON_KEY = Deno.env.get("SUPABASE_ANON_KEY")!;

const ALLOWED_ORIGINS = [
  "https://gmadrid.github.io",
  "http://localhost:8080",
  "http://localhost:3141",
];

function corsHeaders(req: Request) {
  const origin = req.headers.get("Origin") ?? "";
  return {
    "Access-Control-Allow-Origin": ALLOWED_ORIGINS.includes(origin)
      ? origin
      : ALLOWED_ORIGINS[0],
    "Access-Control-Allow-Headers":
      "authorization, x-client-info, apikey, content-type",
    "Access-Control-Allow-Methods": "POST, OPTIONS",
  };
}

Deno.serve(async (req) => {
  // Handle CORS preflight
  if (req.method === "OPTIONS") {
    return new Response(null, { headers: corsHeaders(req) });
  }

  try {
    // Verify the user's JWT
    const authHeader = req.headers.get("Authorization");
    if (!authHeader) {
      return new Response(JSON.stringify({ error: "No auth header" }), {
        status: 401,
        headers: { ...corsHeaders(req), "Content-Type": "application/json" },
      });
    }

    // Create client authenticated as the calling user
    const supabase = createClient(SUPABASE_URL, SUPABASE_ANON_KEY, {
      global: { headers: { Authorization: authHeader } },
    });

    const {
      data: { user },
      error: authError,
    } = await supabase.auth.getUser();

    if (authError || !user) {
      return new Response(
        JSON.stringify({ error: "Unauthorized", detail: authError?.message }),
        {
          status: 401,
          headers: { ...corsHeaders(req), "Content-Type": "application/json" },
        },
      );
    }

    // Fetch user's recent answer logs (RLS filters by user automatically)
    const { data: logs, error: logsError } = await supabase
      .from("answer_log")
      .select("table_index, correct, player_action, correct_action, created_at")
      .eq("user_id", user.id)
      .order("created_at", { ascending: false })
      .limit(500);

    if (logsError) {
      return new Response(
        JSON.stringify({ error: "Failed to fetch logs" }),
        {
          status: 500,
          headers: { ...corsHeaders(req), "Content-Type": "application/json" },
        },
      );
    }

    // Fetch user's deck state
    const { data: deckData } = await supabase
      .from("user_deck")
      .select("study_mode, deck")
      .eq("user_id", user.id)
      .single();

    // Build the coaching prompt
    const totalAnswers = logs?.length ?? 0;
    const correctAnswers = logs?.filter((l: any) => l.correct).length ?? 0;
    const accuracy =
      totalAnswers > 0
        ? ((correctAnswers / totalAnswers) * 100).toFixed(1)
        : "0";

    // Group mistakes by table_index
    const mistakes: Record<string, { wrong: number; total: number }> = {};
    for (const log of logs ?? []) {
      if (!mistakes[log.table_index]) {
        mistakes[log.table_index] = { wrong: 0, total: 0 };
      }
      mistakes[log.table_index].total++;
      if (!log.correct) {
        mistakes[log.table_index].wrong++;
      }
    }

    const troubleSpots = Object.entries(mistakes)
      .filter(([_, v]) => v.wrong > 0)
      .sort((a, b) => b[1].wrong - a[1].wrong)
      .slice(0, 15)
      .map(
        ([idx, v]) =>
          `${idx}: wrong ${v.wrong}/${v.total} (${((1 - v.wrong / v.total) * 100).toFixed(0)}% accuracy)`,
      )
      .join("\n");

    // Group by day for session history
    const byDay: Record<string, { total: number; correct: number }> = {};
    for (const log of logs ?? []) {
      const day = log.created_at.split("T")[0];
      if (!byDay[day]) byDay[day] = { total: 0, correct: 0 };
      byDay[day].total++;
      if (log.correct) byDay[day].correct++;
    }
    const sessionHistory = Object.entries(byDay)
      .sort((a, b) => b[0].localeCompare(a[0]))
      .slice(0, 7)
      .map(
        ([day, v]) =>
          `${day}: ${v.correct}/${v.total} (${((v.correct / v.total) * 100).toFixed(0)}%)`,
      )
      .join("\n");

    const studyMode = deckData?.study_mode ?? "unknown";

    const systemPrompt = `You are a blackjack basic strategy coach. The user is practicing memorizing the basic strategy chart using a spaced repetition trainer. Your role is to:

1. Analyze their performance data and identify patterns in their mistakes
2. Explain WHY specific strategy decisions are correct using probability reasoning
3. Suggest what to focus on next
4. Be encouraging but direct

Keep your response concise (under 300 words). Use short paragraphs. Reference specific hands when discussing mistakes.

IMPORTANT: Use ONLY the strategy tables below when coaching. Do NOT use any other basic strategy — the user is training with these exact tables.

Table index format: "type:row,col" where col is dealer's up card (1=Ace, 2-10).
Actions: H=Hit, S=Stand, Dh=Double (Hit if can't), Ds=Double (Stand if can't), P=Split, Pd=Split (DAS only, otherwise don't split).

## Hard Totals
Columns: 2  3  4  5  6  7  8  9  T  A
8-:       H  H  H  H  H  H  H  H  H  H
9:        H  Dh Dh Dh Dh H  H  H  H  H
10:       Dh Dh Dh Dh Dh Dh Dh Dh H  H
11:       Dh Dh Dh Dh Dh Dh Dh Dh Dh Dh
12:       H  H  S  S  S  H  H  H  H  H
13:       S  S  S  S  S  H  H  H  H  H
14:       S  S  S  S  S  H  H  H  H  H
15:       S  S  S  S  S  H  H  H  H  H
16:       S  S  S  S  S  H  H  H  H  H
17+:      S  S  S  S  S  S  S  S  S  S

## Soft Totals
Columns: 2  3  4  5  6  7  8  9  T  A
A,2 (13): H  H  H  Dh Dh H  H  H  H  H
A,3 (14): H  H  H  Dh Dh H  H  H  H  H
A,4 (15): H  H  Dh Dh Dh H  H  H  H  H
A,5 (16): H  H  Dh Dh Dh H  H  H  H  H
A,6 (17): H  Dh Dh Dh Dh H  H  H  H  H
A,7 (18): Ds Ds Ds Ds Ds S  S  H  H  H
A,8 (19): S  S  S  S  Ds S  S  S  S  S
A,9 (20): S  S  S  S  S  S  S  S  S  S
A,T (21): S  S  S  S  S  S  S  S  S  S

## Pair Splitting
Columns: 2  3  4  5  6  7  8  9  T  A
A,A:      P  P  P  P  P  P  P  P  P  P
2,2:      Pd Pd P  P  P  P  -  -  -  -
3,3:      Pd Pd P  P  P  P  -  -  -  -
4,4:      -  -  -  Pd Pd -  -  -  -  -
5,5:      -  -  -  -  -  -  -  -  -  -
6,6:      Pd P  P  P  P  -  -  -  -  -
7,7:      P  P  P  P  P  P  -  -  -  -
8,8:      P  P  P  P  P  P  P  P  P  P
9,9:      P  P  P  P  P  -  P  P  -  -
T,T:      -  -  -  -  -  -  -  -  -  -

("-" means do NOT split; use the hard total instead. "Pd" means split only with DAS.)
Rules: 6-deck shoe, dealer stands on soft 17, double after split allowed (DAS), no surrender.`;

    const userMessage = `Here's my practice data:

**Overall:** ${correctAnswers}/${totalAnswers} correct (${accuracy}%)
**Current mode:** ${studyMode}

**Trouble spots (most missed):**
${troubleSpots || "None yet"}

**Recent sessions:**
${sessionHistory || "No sessions yet"}

Based on this data, give me coaching advice: what patterns do you see in my mistakes, why are those decisions correct, and what should I focus on next?`;

    // Call Claude API
    const claudeResponse = await fetch(
      "https://api.anthropic.com/v1/messages",
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "x-api-key": ANTHROPIC_API_KEY,
          "anthropic-version": "2023-06-01",
        },
        body: JSON.stringify({
          model: "claude-haiku-4-5-20251001",
          max_tokens: 1024,
          messages: [
            {
              role: "user",
              content: userMessage,
            },
          ],
          system: systemPrompt,
        }),
      },
    );

    if (!claudeResponse.ok) {
      console.error("Claude API error:", await claudeResponse.text());
      return new Response(
        JSON.stringify({ error: "Coaching service temporarily unavailable." }),
        {
          status: 502,
          headers: { ...corsHeaders(req), "Content-Type": "application/json" },
        },
      );
    }

    const claudeData = await claudeResponse.json();
    const coaching =
      claudeData.content?.[0]?.text ?? "No coaching available.";

    return new Response(JSON.stringify({ coaching }), {
      headers: { ...corsHeaders(req), "Content-Type": "application/json" },
    });
  } catch (err) {
    console.error("Coaching function error:", err);
    return new Response(
      JSON.stringify({ error: "An unexpected error occurred." }),
      {
        status: 500,
        headers: { ...corsHeaders(req), "Content-Type": "application/json" },
      },
    );
  }
});
