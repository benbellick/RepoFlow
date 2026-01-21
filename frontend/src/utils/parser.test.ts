import { describe, it, expect } from "vitest";
import { parseGitHubUrl } from "./parser";

describe("parseGitHubUrl", () => {
  it("correctly parses a standard GitHub URL", () => {
    const url = "https://github.com/facebook/react";
    const result = parseGitHubUrl(url);
    expect(result).toEqual({ owner: "facebook", repo: "react" });
  });

  it("handles trailing slashes", () => {
    const url = "https://github.com/rust-lang/rust/";
    const result = parseGitHubUrl(url);
    expect(result).toEqual({ owner: "rust-lang", repo: "rust" });
  });

  it("returns null for non-github URLs", () => {
    const url = "https://gitlab.com/owner/repo";
    const result = parseGitHubUrl(url);
    expect(result).toBeNull();
  });

  it("returns null for incomplete URLs", () => {
    const url = "https://github.com/facebook";
    const result = parseGitHubUrl(url);
    expect(result).toBeNull();
  });

  it("returns null for invalid strings", () => {
    const url = "not-a-url";
    const result = parseGitHubUrl(url);
    expect(result).toBeNull();
  });
});
