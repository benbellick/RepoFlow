# RepoFlow

A tool to measure how efficiently pull requests flow through open source projects.

## The Problem

When you want to contribute to an open source project, it's hard to know if your PR will actually get reviewed and merged, or if it'll sit in limbo forever. Some projects are underwater with contributions they can't process. Others are healthy and responsive. There's no easy way to tell the difference.

Similarly, maintainers often don't have visibility into whether their project is keeping up with contribution interest, or if they're falling behind and building up a backlog.

## The Solution

**RepoFlow** visualizes PR flow efficiency by tracking two simple metrics over time:
- **PRs opened**: How many contributions are coming in
- **PRs merged**: How many contributions are getting processed

The gap between these (the "spread") tells you whether a project can keep up with contributor interest.

**What different patterns mean:**
- **Small spread, high volume**: Healthy, active project keeping up with contributions
- **Growing spread**: Backlog building, maintainers getting overwhelmed
- **Tight spread, declining volume**: Mature/stable project with less activity
- **Wide spread, high volume**: Popular project that can't scale maintenance

## Inspiration

The idea was inspired by financial market concepts like liquidity and bid-ask spreads. In markets, a "liquid" market means transactions flow smoothly with tight spreads. An "illiquid" market has high friction and wide spreads.

The same intuition applies to OSS: a "liquid" project processes PRs efficiently (tight spread), while a "frozen" project has PRs piling up (wide spread).

**Note:** While the metaphor is useful, we're not literally modeling markets. We measure flow rates (PRs/time) rather than order book mechanics, since the same PR doesn't necessarily appear in both "opened" and "merged" within a single time window.

## How It Works

### Data Collection
- User pastes a GitHub repo URL
- Frontend fetches PR data from GitHub's public API (no auth required for basic usage)
- Calculates metrics using rolling time windows

### Metrics Explained

**Rolling Window Approach:**
For each date point, we count PRs in a trailing window. For example, with a 30-day window:
- Point on Jan 15: Count PRs opened from Dec 16-Jan 15, and PRs merged from Dec 16-Jan 15
- Point on Jan 16: Count PRs opened from Dec 17-Jan 16, and PRs merged from Dec 17-Jan 16

This gives us two lines showing flow rates over time. The vertical distance between them is the spread.

**Primary Metrics:**
- **PRs Opened** (trailing window count)
- **PRs Merged** (trailing window count)
- **The Spread** (opened - merged)
- **Merge Rate** (merged / opened as percentage)
- **Volume** (total activity)

**Key Point:** We're measuring flow rates, not tracking individual PRs. A PR opened on Dec 20 and merged on Jan 10 will appear in both metrics in different time windows. This is fine - we want to know if inflow rate exceeds outflow rate on average.

### Visualization

Two line graph showing:
- Blue line: PRs opened over time (30-day rolling count)
- Pink line: PRs merged over time (30-day rolling count)
- The gap between them: The spread

Plus stats cards showing current values and trend analysis (is spread widening or tightening?).

## Technical Implementation

### V1 Scope

**Keep it dead simple:**
- Pure frontend, no backend
- Client-side GitHub API calls
- No caching or persistence initially
- Accept rate limits as constraint
- Single repo analysis at a time

### Tech Stack

**Framework:** TypeScript + React
- Considered PureScript and Rust/WASM but prioritizing shipping speed
- TypeScript is pragmatic for v1

**Design System:** Neo-brutalism using neobrutalism.dev
- Pre-built component library with the aesthetic we want
- Includes chart components already styled correctly
- React + Tailwind based
- Thick borders, hard shadows, bold colors, chunky typography

**Deployment:** Vercel
- Git-based deployment
- No secrets in repo
- Zero lock-in (just static files)
- Can switch to Cloudflare Pages or self-host anytime

**State Management:**
- V1: No persistence (just analyze and display)
- Future: localStorage or Vercel KV if we want to save/compare multiple repos

### User Flow

1. User lands on site with examples of healthy projects pre-loaded (React, Kubernetes, etc.)
2. User pastes their own GitHub repo URL
3. Parse owner/repo from URL
4. Fetch PR data from GitHub API
5. Calculate rolling window metrics
6. Display visualization with stats cards and trend analysis

### GitHub API Integration

- Use GitHub's public REST API
- No authentication needed for public repos (60 requests/hour)
- Can add optional user token for higher limits
- Fetch PRs with their created_at and merged_at timestamps
- Filter and count client-side based on date ranges

## Future Enhancements

**Issue→PR Conversion Analysis:**
Track issues opened vs PRs that address them. Issues without PRs are like "fake demand signals" - lots of talk but no action. This measures:
- Signal-to-noise ratio
- Contributor activation (can people go from "I want this" to "I built this"?)
- Community health

**Three-line visualization:**
- Issues opened (stated demand)
- PRs opened (actual work being done)
- PRs merged (completed work)

Gaps between the lines tell different stories about project health.

**Contributor Metrics:**
- Number of unique contributors (more participants = healthier ecosystem)
- Contributor concentration (are contributions spread out or dominated by a few people?)
- New contributors over time (is the project attracting fresh people?)

**Advanced Analysis:**
- Time-to-merge velocity distribution
- Review response time
- Spread volatility/stability
- Comparison mode (analyze multiple repos side by side)

## Project Structure

```
├── src/
│   ├── components/
│   │   ├── RepoInput.tsx
│   │   ├── StatsCards.tsx
│   │   ├── FlowChart.tsx
│   │   └── TrendAnalysis.tsx
│   ├── utils/
│   │   ├── github.ts        # API calls
│   │   ├── metrics.ts       # Calculations
│   │   └── parser.ts        # URL parsing
│   └── App.tsx
├── package.json
└── README.md
```

## Open Questions for V1

- What time window(s) to show? (7/30/90 day? User selectable?)
- Which example repos to hardcode on landing page?
- Do we show multiple window sizes simultaneously or let user toggle?
- How to handle repos with very few PRs (edge case UX)?

## Success Metrics

How do we know if this is useful?
- People actually use it to evaluate projects before contributing
- Maintainers use it to monitor their project health
- Someone builds tooling on top of it (badge for README, API, etc.)
- Sparks conversations about project health in OSS communities
