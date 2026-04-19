// Mock data + content for the chat prototype
window.PROVIDERS = [
  {
    id: 'ollama',
    name: 'Ollama',
    desc: 'Local',
    models: [
      { id: 'llama3.1:70b', name: 'llama3.1:70b', desc: 'General purpose, balanced', tags: ['70B', 'local'] },
      { id: 'qwen2.5-coder:32b', name: 'qwen2.5-coder:32b', desc: 'Code-focused, fast on M-series', tags: ['32B', 'code'] },
      { id: 'mistral-nemo:12b', name: 'mistral-nemo:12b', desc: 'Lightweight, multilingual', tags: ['12B'] },
      { id: 'phi4:14b', name: 'phi4:14b', desc: 'Reasoning-heavy small model', tags: ['14B'] },
    ],
  },
  {
    id: 'deepseek',
    name: 'DeepSeek Direct',
    desc: 'API',
    models: [
      { id: 'deepseek-chat', name: 'deepseek-chat', desc: 'Fast general chat (V3)', tags: ['cloud'] },
      { id: 'deepseek-reasoner', name: 'deepseek-reasoner', desc: 'Chain-of-thought (R1)', tags: ['reasoning'] },
    ],
  },
  {
    id: 'openrouter',
    name: 'OpenRouter',
    desc: 'API',
    models: [
      { id: 'anthropic/claude-sonnet-4.5', name: 'claude-sonnet-4.5', desc: 'Anthropic', tags: ['cloud'] },
      { id: 'openai/gpt-5', name: 'gpt-5', desc: 'OpenAI', tags: ['cloud'] },
      { id: 'meta/llama-3.3-405b', name: 'llama-3.3-405b', desc: 'Meta', tags: ['405B'] },
      { id: 'x-ai/grok-4', name: 'grok-4', desc: 'xAI', tags: ['cloud'] },
    ],
  },
];

window.PERSONAS = [
  {
    id: 'cofounder',
    name: 'Technical Co-founder',
    color: 'var(--persona-tech)',
    desc: 'Pragmatic, ship-it mindset. Weighs tradeoffs against time and runway.',
  },
  {
    id: 'engineer',
    name: 'Sr. Engineer',
    color: 'var(--persona-eng)',
    desc: 'Rigorous, code-first. Will push back on shaky designs.',
  },
  {
    id: 'research',
    name: 'Research Assistant',
    color: 'var(--persona-research)',
    desc: 'Cites sources, synthesises across papers, surfaces unknowns.',
  },
];

window.CONVERSATIONS = [
  // Today
  { id: 'c1', title: 'Postgres connection pool tuning', when: '2:14 PM', group: 'Today', active: true, persona: 'engineer' },
  { id: 'c2', title: 'Pricing page draft v3', when: '11:02 AM', group: 'Today', persona: 'cofounder' },
  { id: 'c3', title: 'Comparing vector DBs for RAG', when: '9:48 AM', group: 'Today', persona: 'research' },
  // Yesterday
  { id: 'c4', title: 'Onboarding flow rewrite', when: 'Yesterday', group: 'Yesterday', persona: 'cofounder' },
  { id: 'c5', title: 'Diagnose slow CI on the staging branch', when: 'Yesterday', group: 'Yesterday', persona: 'engineer' },
  { id: 'c6', title: 'Series A deck — narrative arc', when: 'Yesterday', group: 'Yesterday', persona: 'cofounder' },
  // Earlier
  { id: 'c7', title: 'Differential privacy tradeoffs', when: 'Mon', group: 'Earlier', persona: 'research' },
  { id: 'c8', title: 'Rust vs Go for the ingest worker', when: 'Mon', group: 'Earlier', persona: 'engineer' },
  { id: 'c9', title: 'Hiring rubric for first 3 engineers', when: 'Sun', group: 'Earlier', persona: 'cofounder' },
  { id: 'c10', title: 'Survey of small-model fine-tuning', when: 'Sat', group: 'Earlier', persona: 'research' },
  { id: 'c11', title: 'Local-first sync: CRDT options', when: 'Fri', group: 'Earlier', persona: 'engineer' },
];

// Pre-built thread for conv c1 — a realistic technical exchange
// "older" messages get loaded on infinite scroll
window.OLDER_MESSAGES = [
  {
    id: 'm-old-1',
    role: 'user',
    when: '1:41 PM',
    content: [{ type: 'p', text: "We're seeing connection storms on the primary every time the worker fleet restarts. Pool size is 200 across 12 workers, pgbouncer in front. What's the actual right way to think about pool sizing here?" }],
  },
  {
    id: 'm-old-2',
    role: 'assistant',
    when: '1:42 PM',
    persona: 'engineer',
    model: 'qwen2.5-coder:32b',
    provider: 'ollama',
    content: [
      { type: 'p', text: "Short answer: 200 per worker × 12 workers = 2,400 potential backend connections, which Postgres will hate. The right mental model is **transactions per second × average transaction time**, not requests per second." },
      { type: 'p', text: "A useful starting heuristic from the pgbouncer docs:" },
      { type: 'code', lang: 'text', code: 'pool_size = ((core_count * 2) + effective_spindle_count)\n// for a 16-core box with SSDs: ~34 connections is plenty' },
      { type: 'p', text: "With pgbouncer in transaction pooling mode, your application-side pool can be much larger than the upstream pool. The shape that usually works:" },
      { type: 'ul', items: [
        "App pool: generous (say 50/worker) — these are cheap pgbouncer client connections",
        "pgbouncer → Postgres: small (20–40 total, not per-worker)",
        "Set `server_idle_timeout` aggressively (60s) so idle backends release",
      ]},
    ],
  },
  {
    id: 'm-old-3',
    role: 'user',
    when: '1:48 PM',
    content: [{ type: 'p', text: "Got it. We're already in transaction pooling mode. The problem is the *startup* — when 12 workers come up at once they all try to fill their pool eagerly. Should I just lazy-init?" }],
  },
  {
    id: 'm-old-4',
    role: 'assistant',
    when: '1:48 PM',
    persona: 'engineer',
    model: 'qwen2.5-coder:32b',
    provider: 'ollama',
    content: [
      { type: 'p', text: "Lazy init is one lever, but the cleaner fix is to **decouple pool capacity from pool warmth**. You want the worker to be ready to handle a burst without paying the connection cost on the first request." },
      { type: 'p', text: "Three things that tend to help in combination:" },
      { type: 'ol', items: [
        "Set `min_connections=1, max_connections=50` instead of fixed-size — most pools support this",
        "Add jittered startup delay (0–5s) so workers don't synchronize their pool fill",
        "Have pgbouncer be the actual gatekeeper, not your app — set its `max_client_conn` high but `default_pool_size` strict",
      ]},
      { type: 'p', text: "Here's roughly what your pool config should look like in a Node service using `pg`:" },
      { type: 'code', lang: 'js', code: "import { Pool } from 'pg';\n\nconst pool = new Pool({\n  host: process.env.PGBOUNCER_HOST,\n  port: 6432,\n  // App-side pool — talks to pgbouncer, NOT postgres\n  min: 2,\n  max: 50,\n  idleTimeoutMillis: 30_000,\n  connectionTimeoutMillis: 5_000,\n  // Required when pgbouncer is in transaction mode\n  statement_timeout: 30_000,\n});\n\n// Warm slowly\nconst jitter = Math.random() * 5000;\nsetTimeout(() => pool.connect().then(c => c.release()), jitter);" },
    ],
  },
];

window.RECENT_MESSAGES = [
  {
    id: 'm1',
    role: 'user',
    when: '2:11 PM',
    content: [{ type: 'p', text: "Last question — does PgBouncer's `transaction` mode break anything in our setup? We use prepared statements pretty heavily in the order service." }],
  },
  {
    id: 'm2',
    role: 'assistant',
    when: '2:12 PM',
    persona: 'engineer',
    model: 'qwen2.5-coder:32b',
    provider: 'ollama',
    content: [
      { type: 'p', text: "Yes — this is the classic gotcha. In transaction pooling mode, **prepared statements don't survive across transactions** because each transaction may land on a different backend connection." },
      { type: 'p', text: "You have three reasonable options:" },
      { type: 'ol', items: [
        "Disable prepared statements client-side (simplest, ~5-15% slower)",
        "Use pgbouncer 1.21+ which has experimental prepared-statement support (`max_prepared_statements`)",
        "Switch the order service specifically to session pooling on a separate pgbouncer pool",
      ]},
      { type: 'p', text: "For a Node app with `pg`, the disable looks like:" },
      { type: 'code', lang: 'js', code: "// Force unnamed statements — pg uses prepared statements by default\nconst result = await client.query({\n  text: 'SELECT * FROM orders WHERE id = $1',\n  values: [orderId],\n  // No `name` field = no prepared statement cache\n});" },
      { type: 'p', text: "Honestly, given you've already had one connection storm, I'd go with option 3. Run the order service against a dedicated pgbouncer pool in session mode (small — maybe 30 connections), and let everything else use transaction pooling. You get the best of both." },
    ],
  },
];
