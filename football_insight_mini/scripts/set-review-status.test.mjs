import { describe, expect, test } from "bun:test";

import { setReviewStatus } from "./set-review-status.mjs";

const config = { apiKey: "secret-key", baseUrl: "https://match.oryjk.cn/api/v1", projectCode: "football_insight_mini", skip: false };

function fetchStub(responses) {
  const calls = [];
  return {
    calls,
    fetchImpl: async (url, init) => {
      calls.push({ url, init });
      const next = responses.shift() ?? { ok: true, status: 200, body: {} };
      return { ok: next.ok, status: next.status, json: async () => next.body, text: async () => JSON.stringify(next.body) };
    },
  };
}

describe("setReviewStatus", () => {
  test("PUTs the project, version and approval with the static api key", async () => {
    const stub = fetchStub([{ ok: true, status: 200, body: { is_reviewing: false } }]);
    await setReviewStatus({ config, version: "1.0.76", isReviewing: false, fetchImpl: stub.fetchImpl });
    expect(stub.calls).toHaveLength(1);
    expect(stub.calls[0].url).toBe("https://match.oryjk.cn/api/v1/mini-review/review-status");
    expect(stub.calls[0].init.method).toBe("PUT");
    expect(stub.calls[0].init.headers["X-Api-Key"]).toBe("secret-key");
    expect(JSON.parse(stub.calls[0].init.body)).toEqual({
      project_code: "football_insight_mini",
      version: "1.0.76",
      is_reviewing: false,
    });
  });

  test("fails loudly with the version and a manual fallback when the registry rejects the call", async () => {
    const stub = fetchStub([{ ok: false, status: 404, body: { error: "not registered" } }]);
    await expect(
      setReviewStatus({ config, version: "1.0.76", isReviewing: false, fetchImpl: stub.fetchImpl }),
    ).rejects.toThrow(/1\.0\.76.*404|404.*1\.0\.76/s);
  });

  test("requires the api key and never leaks it in errors", async () => {
    const stub = fetchStub([]);
    await expect(
      setReviewStatus({ config: { ...config, apiKey: "" }, version: "1.0.76", isReviewing: false, fetchImpl: stub.fetchImpl }),
    ).rejects.toThrow("MINI_REVIEW_API_KEY");
    expect(stub.calls).toHaveLength(0);
    const failing = fetchStub([{ ok: false, status: 500, body: {} }]);
    await setReviewStatus({ config, version: "1.0.76", isReviewing: false, fetchImpl: failing.fetchImpl }).catch((error) => {
      expect(String(error.message)).not.toContain("secret-key");
    });
  });

  test("rejects malformed versions before calling the registry", async () => {
    const stub = fetchStub([]);
    await expect(setReviewStatus({ config, version: "abc", isReviewing: false, fetchImpl: stub.fetchImpl })).rejects.toThrow();
    expect(stub.calls).toHaveLength(0);
  });
});
