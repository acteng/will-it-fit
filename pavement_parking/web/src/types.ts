export type Mode = "roads" | "lad-summary" | "ca-summary" | "census-area";

export interface Filters {
  useRating: "rating_exc_pavements" | "rating_change";
  showRatings: {
    green: boolean;
    amber: boolean;
    red: boolean;
    no_change: boolean;
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
  useRating: "rating_exc_pavements",
  showRatings: {
    green: true,
    amber: true,
    red: true,
    no_change: true,
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
  black: "#000000",
};
