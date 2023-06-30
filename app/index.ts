import * as p5 from 'p5';
import App from './app';

import "../public/main.css"
import { addSliderListener } from './addSliderListener';
import { FPS } from './cfg';

const app = new App();

app.setup();

// p.draw = () => {
//   p.clear(0, 0, 0, 0);
//   app.draw();
//   app.update();
// }

setInterval((app) => {
  app.draw();
  app.update();
}, 1000 / FPS, app);

addSliderListener('#distanceSlider', '#distanceInput')
addSliderListener('#electricFieldSlider', '#electricFieldInput')
addSliderListener('#suppressionSlider', '#suppressionInput')
addSliderListener('#velocitySlider', '#velocityInput')
addSliderListener('#electronsSlider', '#electronsInput')
addSliderListener('#volumeSlider', '#volumeInput')

const resetButton = document.getElementById('resetButton') as HTMLButtonElement;
resetButton.addEventListener('click', () => {
  app.resetSimulation();
});