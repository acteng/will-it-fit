export type Mode = "roads" | "lad-summary" | "ca-summary" | "census-area";

export interface Filters {
  scenario: "U" | "X" | "Y" | "Z";
  showRatings: {
    green: boolean;
    amber: boolean;
    red: boolean;
  };
  showClasses: {
    A: boolean;
    B: boolean;
    C: boolean;
    Unclassified: boolean;
  };
  showDirections: {
    both: boolean;
    "one-way": boolean;
  };
}

export const defaultFilters: Filters = {
  scenario: "U",
  showRatings: {
    green: true,
    amber: true,
    red: true,
  },
  showClasses: {
    A: true,
    B: true,
    C: true,
    Unclassified: true,
  },
  showDirections: {
    both: true,
    "one-way": true,
  },
};

export const colors = {
  green: "#006853",
  amber: "#ffd833",
  red: "#b73d25",
};

export const scenarios = ["U", "X", "Y", "Z"] as const;
export const ratings = ["red", "amber", "green"] as const;
