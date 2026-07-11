import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it } from "vitest";

import { useSearchStore } from "@/stores/search";

describe("search store", () => {
    beforeEach(() => setActivePinia(createPinia()));

    it("records repeated submissions of the same query", () => {
        const search = useSearchStore();

        search.submit("vue");
        search.submit("vue");

        expect(search.condition).toBe("vue");
        expect(search.requestRevision).toBe(2);
    });
});
