/**
 * API client tests: error mapping and URL construction (fetch is mocked).
 */
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  ApiError,
  fetchOverview,
  fetchTrains,
  TIME_RANGES,
} from "../src/lib/api";

describe("api client", () => {
  const fetchMock = vi.fn();

  beforeEach(() => {
    vi.stubGlobal("fetch", fetchMock);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    fetchMock.mockReset();
  });

  it("exposes the four documented ranges", () => {
    expect(TIME_RANGES.map((r) => r.value)).toEqual(["1h", "6h", "24h", "7d"]);
  });

  it("requests /overview with the range parameter", async () => {
    fetchMock.mockResolvedValue(
      new Response(JSON.stringify({ health: { prometheus_reachable: true } }), {
        status: 200,
      }),
    );
    await fetchOverview("6h");
    const [url, init] = fetchMock.mock.calls[0];
    expect(url).toBe("/api/overview?range=6h");
    expect(init.headers.Accept).toBe("application/json");
  });

  it("surfaces structured API errors", async () => {
    fetchMock.mockResolvedValue(
      new Response(
        JSON.stringify({
          error: {
            code: "invalid_time_range",
            message: "invalid time range: 3w",
          },
        }),
        { status: 400 },
      ),
    );
    await expect(fetchOverview("3w" as never)).rejects.toMatchObject({
      code: "invalid_time_range",
      status: 400,
    });
  });

  it("maps network failures to a network_error ApiError", async () => {
    fetchMock.mockRejectedValue(new TypeError("network down"));
    const err = await fetchTrains("1h").catch((e: unknown) => e);
    expect(err).toBeInstanceOf(ApiError);
    expect((err as ApiError).code).toBe("network_error");
  });

  it("falls back to a generic message for non-JSON error bodies", async () => {
    fetchMock.mockResolvedValue(new Response("boom", { status: 502 }));
    const err = await fetchTrains("1h").catch((e: unknown) => e);
    expect((err as ApiError).status).toBe(502);
    expect((err as ApiError).code).toBe("http_error");
  });
});
