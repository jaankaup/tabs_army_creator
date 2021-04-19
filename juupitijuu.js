//import init, { add} from './pkg/wasm_game_of_life.js';
import init from './pkg/wasm_game_of_life.js';

//function run() {
//  const result = add(1, 2);
//  console.log(`1 + 2 = ${result}`);
//  if (result !== 3)
//    throw new Error("wasm addition doesn't work!");
//}


async function initialize_wasm() {
  await init();
  console.log("initialize_wasm ok.");
}

async function loop() {
  await initialize_wasm();
  // run();
}

loop();
