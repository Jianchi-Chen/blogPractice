import { defineStore } from "pinia";

export const useSearchStore = defineStore("search", {
    state: () => ({
        condition: "",
        requestRevision: 0,
    }),

    actions: {
        submit(value: string) {
            this.condition = value;
            this.requestRevision += 1;
        },
    },
});
