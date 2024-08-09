export interface Filters {
  useRating: "average_rating" | "minimum_rating";
  showRatings: {
    green: boolean;
    amber: boolean;
    red: boolean;
    no_change: boolean;
  };
  showClasses: {
    "A Road": boolean;
    "B Road": boolean;
    "Classified Unnumbered": boolean;
    Unclassified: boolean;
    Unknown: boolean;
    "Not Classified": boolean;
  };
  showDirections: {
    both: boolean;
    "one-way": boolean;
  };
}

export const defaultFilters: Filters = {
  useRating: "average_rating",
  showRatings: {
    green: true,
    amber: true,
    red: true,
    no_change: true,
  },
  showClasses: {
    "A Road": true,
    "B Road": true,
    "Classified Unnumbered": true,
    Unclassified: true,
    Unknown: true,
    "Not Classified": true,
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
