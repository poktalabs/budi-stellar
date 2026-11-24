import React from "react";

// Stacks an invisible copy of the final content under the live content in one grid cell, so
// typing, counting or swapping text never changes the layout width (the visible remake bugs).
export const Reserve: React.FC<{ final: React.ReactNode; children: React.ReactNode; align?: "left" | "right" | "center" }> = ({
  final,
  children,
  align = "left",
}) => (
  <span style={{ display: "inline-grid", textAlign: align }}>
    <span style={{ gridArea: "1 / 1", visibility: "hidden" }} aria-hidden>
      {final}
    </span>
    <span style={{ gridArea: "1 / 1" }}>{children}</span>
  </span>
);
