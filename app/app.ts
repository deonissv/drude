import * as p5 from "p5";
import { CANVAS_HEIGHT, CANVAS_WIDTH, ELECTRON_COLOR, ELECTRON_M, ELECTRON_Q, ELECTRON_RADIUS, FPS, ION_COLOR, ION_RADIUS, TIME_SCALE, TO_CM_POW_2, TO_MM_POW_2, TO_UM, UPDATE_EVERY_N, VOLUME_SCALE } from "./cfg";
import * as utils from "utils";
utils;

export default class App {
  ctx: CanvasRenderingContext2D;
  cs: utils.CrystalStructure;
  ions: utils.Ion[];

  sliderText: p5.Element;
  fieldStrengthSlider: HTMLInputElement;
  suppressionSlider: HTMLInputElement;
  distanceSlider: HTMLInputElement;
  velocitySlider: HTMLInputElement;
  electronsSlider: HTMLInputElement;
  volumeInput: HTMLInputElement;
  electronsLeft: HTMLSpanElement;
  electronsRight: HTMLSpanElement;
  electronsDifference: HTMLSpanElement;
  currentValue: HTMLSpanElement;
  avgTime: HTMLSpanElement;
  driftVelocity: HTMLSpanElement;
  electronMobility: HTMLSpanElement;
  currentDensity: HTMLSpanElement;
  leftBorderCounter: number;
  rightBorderCounter: number;

  acc: number;
  supp: number;
  sinceCountersUpdate: number;


  constructor() {
    const canvas = document.getElementById('canvas') as HTMLCanvasElement;
    canvas.setAttribute('width', `${CANVAS_WIDTH}px`);
    canvas.setAttribute('height', `${CANVAS_HEIGHT}px`);

    this.ctx = canvas.getContext('2d');

    this.leftBorderCounter = 0;
    this.rightBorderCounter = 0;

    this.distanceSlider = document.getElementById('distanceSlider') as HTMLInputElement;
    this.fieldStrengthSlider = document.getElementById('electricFieldSlider') as HTMLInputElement;
    this.suppressionSlider = document.getElementById('suppressionSlider') as HTMLInputElement;
    this.electronsSlider = document.getElementById('electronsSlider') as HTMLInputElement;
    this.velocitySlider = document.getElementById('velocitySlider') as HTMLInputElement;
    this.volumeInput = document.getElementById('volumeInput') as HTMLInputElement;

    this.electronsLeft = document.getElementById('electronsLeft') as HTMLSpanElement;
    this.electronsRight = document.getElementById('electronsRight') as HTMLSpanElement;
    this.electronsDifference = document.getElementById('electronsDifference') as HTMLSpanElement;
    this.currentValue = document.getElementById('currentValue') as HTMLSpanElement;
    this.avgTime = document.getElementById('avgTime') as HTMLSpanElement;
    this.driftVelocity = document.getElementById('driftVelocity') as HTMLSpanElement;
    this.electronMobility = document.getElementById('electronMobility') as HTMLSpanElement;
    this.currentDensity = document.getElementById('currentDensity') as HTMLSpanElement;
  }

  setup() {
    const fs = +this.fieldStrengthSlider.value;
    const supp = +this.suppressionSlider.value;

    this.acc = fs * ELECTRON_M / ELECTRON_Q;
    this.supp = supp;
    this.sinceCountersUpdate = 0;
    this.resetSimulation();
  }

  drawCircle(x: number, y: number, radius: number, fill: string, stroke?: string, strokeWidth?: number) {
    this.ctx.beginPath()
    this.ctx.arc(x, y, radius, 0, 2 * Math.PI, false)
    if (fill) {
      this.ctx.fillStyle = fill
      this.ctx.fill()
    }
    if (stroke) {
      this.ctx.lineWidth = strokeWidth
      this.ctx.strokeStyle = stroke
      this.ctx.stroke()
    }
  }

  draw() {
    this.ctx.clearRect(0, 0, CANVAS_WIDTH, CANVAS_HEIGHT);

    this.ions.forEach((ion) => {
      this.drawCircle(ion.x, ion.y, ION_RADIUS, ION_COLOR, 'black', 1);
    });

    this.cs.get_electrons().forEach((electron) => {
      this.drawCircle(electron.x, electron.y, ELECTRON_RADIUS, ELECTRON_COLOR);
    });
  }

  resetSimulation() {
    const initVelocity = +this.velocitySlider.value;
    const electronsNumber = +this.electronsSlider.value;
    const distance = +this.distanceSlider.value;
    this.cs = new utils.CrystalStructure(CANVAS_WIDTH, CANVAS_HEIGHT, distance, initVelocity, electronsNumber);
    this.ions = this.cs.get_ions();
  }

  update() {
    this.cs.update(this.acc, this.supp);

    if (this.sinceCountersUpdate == UPDATE_EVERY_N) {
      const fs = +this.fieldStrengthSlider.value;
      const supp = +this.suppressionSlider.value;

      this.acc = fs / 10;
      this.supp = supp / 10;

      this.electronsRight.textContent = "" + this.rightBorderCounter;
      this.electronsLeft.textContent = "" + this.leftBorderCounter;
      this.electronsDifference.textContent = "" + (this.rightBorderCounter - this.leftBorderCounter);
      let current = (this.rightBorderCounter - this.leftBorderCounter) * ELECTRON_Q / (UPDATE_EVERY_N / FPS);
      this.currentValue.textContent = "" + current ** 2;
      this.leftBorderCounter = this.cs.elec_left;
      this.rightBorderCounter = this.cs.elec_right;
      this.sinceCountersUpdate = 0;

      const num_electrons = +this.electronsSlider.value;
      const avg_time = this.cs.avg_ticks_between_bounces() * TIME_SCALE / FPS / num_electrons;
      this.avgTime.textContent = "" + avg_time;

      const drift_velocity = ELECTRON_Q * fs * avg_time * TO_UM / ELECTRON_M;
      const electron_mobility = ELECTRON_Q * avg_time * TO_CM_POW_2 / ELECTRON_M;
      const volume = +this.volumeInput.value;
      const current_density = ELECTRON_Q * num_electrons / (volume * VOLUME_SCALE) * electron_mobility * fs / TO_MM_POW_2;

      this.driftVelocity.textContent = "" + drift_velocity.toFixed(2);
      this.electronMobility.textContent = "" + electron_mobility.toFixed(2);
      this.currentDensity.textContent = "" + current_density.toFixed(2);
    }
    this.sinceCountersUpdate += 1;

  }
}