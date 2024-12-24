describe("App", () => {
  beforeEach(() => {
    cy.visit("/");
    // Wait for Monaco editor to load
    cy.get(".monaco-editor", { timeout: 10000 }).should("exist");
  });

  it("shows editor layout", () => {
    cy.get("aside").should("exist");
    cy.get("main").should("exist");
    cy.get(".monaco-editor").should("be.visible");
  });
});
