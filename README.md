# RepoFlow

**RepoFlow** is a tool designed to measure the "liquidity" of open source projects. It visualizes how efficiently pull requests flow through a repository by tracking the rate of incoming contributions versus the rate of merged PRs.

![RepoFlow Preview](https://placehold.co/600x400?text=RepoFlow+Preview) 

## The Problem

When contributing to open source, it's often hard to tell if a project is healthy and responsive or if it's drowning in a backlog. **RepoFlow** solves this by visualizing the "spread" between opened and merged PRs over time.

- **Tight Spread:** The project is processing contributions efficiently (Healthy).
- **Widening Spread:** Contributions are outpacing the maintainers' ability to review and merge (Backlog building).

## Features

- **Flow Visualization:** Interactive line charts showing the volume of opened vs. merged PRs over a rolling 30-day window.
- **Key Metrics:** Instant insights into the "Spread", Merge Rate, and volume trends.
- **Neo-brutalism Design:** A bold, high-contrast UI built with Tailwind CSS.
- **GitHub Integration:** Fetches real-time data directly from the GitHub public API.

## Tech Stack

- **Frontend:** React + TypeScript (Vite)
- **Styling:** Tailwind CSS (Neo-brutalism aesthetic)
- **Charts:** Recharts
- **Icons:** Lucide React

## Getting Started

### Prerequisites

- Node.js (v18 or higher)
- npm

### Installation

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/benbellick/RepoFlow.git
    cd RepoFlow
    ```

2.  **Install dependencies:**
    ```bash
    npm install
    ```

3.  **Set up Environment Variables (Optional but Recommended):**
    To avoid GitHub API rate limits (60 requests/hour for unauthenticated users), create a `.env` file in the root directory and add your GitHub Personal Access Token:

    ```env
    VITE_GITHUB_TOKEN=your_personal_access_token_here
    ```

4.  **Run the development server:**
    ```bash
    npm run dev
    ```

5.  **Open in Browser:**
    Navigate to `http://localhost:5173` to start using RepoFlow.

## Usage

1.  Enter a GitHub repository URL (e.g., `https://github.com/facebook/react`) in the search bar.
2.  Click **Analyze**.
3.  View the flow metrics and charts to evaluate the project's health.

## License

MIT