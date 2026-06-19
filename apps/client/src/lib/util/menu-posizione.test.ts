import { describe, it, expect } from "vitest";
import { posizionaMenu } from "./menu-posizione";

const VW = 1000;
const VH = 800;
const W = 200;
const H = 300;

describe("posizionaMenu", () => {
  it("lascia la posizione invariata quando il menu ci sta", () => {
    expect(posizionaMenu(100, 100, W, H, VW, VH)).toEqual({
      left: 100,
      top: 100,
    });
  });

  it("flippa a sinistra quando sborda a destra", () => {
    // x=900, w=200 → 1100 > 1000 → left = 900 - 200 = 700
    const p = posizionaMenu(900, 100, W, H, VW, VH);
    expect(p.left).toBe(700);
    expect(p.top).toBe(100);
  });

  it("flippa sopra quando sborda in basso", () => {
    // y=700, h=300 → 1000 > 800 → top = 700 - 300 = 400
    const p = posizionaMenu(100, 700, W, H, VW, VH);
    expect(p.left).toBe(100);
    expect(p.top).toBe(400);
  });

  it("flippa su entrambi gli assi nell'angolo in basso a destra", () => {
    const p = posizionaMenu(950, 750, W, H, VW, VH);
    expect(p.left).toBe(750);
    expect(p.top).toBe(450);
  });

  it("clampa al margine minimo vicino al bordo alto-sinistra", () => {
    const p = posizionaMenu(2, 2, W, H, VW, VH);
    expect(p.left).toBe(8);
    expect(p.top).toBe(8);
  });

  it("clampa al margine quando il menu è più grande del viewport", () => {
    const p = posizionaMenu(50, 50, 2000, 2000, VW, VH);
    expect(p.left).toBe(8);
    expect(p.top).toBe(8);
  });

  it("rispetta un margine personalizzato", () => {
    const p = posizionaMenu(900, 100, W, H, VW, VH, 20);
    expect(p.left).toBe(700);
  });
});
