import { Chip8 } from "chip8-wasm";
import { memory } from "chip8-wasm/chip8_bg";

const SCALE = 5;
const SPEED = 8;

// wasm component
const chip8 = Chip8.new();
const width = chip8.width();
const height = chip8.height();

// UI component
const canvas = document.getElementById("chip8-canvas");
canvas.width = width * SCALE;
canvas.height = height * SCALE;
const ctx = canvas.getContext("2d");
const playBtn = document.getElementById("play-btn");
const rom = document.getElementById("rom");

const keymap = {
  49: 0x1, // 1
  50: 0x2, // 2
  51: 0x3, // 3
  52: 0xc, // 4
  81: 0x4, // Q
  87: 0x5, // W
  69: 0x6, // E
  82: 0xd, // R
  65: 0x7, // A
  83: 0x8, // S
  68: 0x9, // D
  70: 0xe, // F
  90: 0xa, // Z
  88: 0x0, // X
  67: 0xb, // C
  86: 0xf // V
};

const getIndex = (row, col) => {
    return row * width + col;
}

const updateScreen = () => {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    const pixels = new Uint8Array(memory.buffer, chip8.get_screen_memory(), width * height);
    for (var row = 0; row < height; row++) {
        for (var col = 0; col < width; col++) {
            const idx = getIndex(row, col);
            if (pixels[idx]) {
                ctx.fillStyle = "#000000";
                ctx.fillRect(col * SCALE, row * SCALE, SCALE, SCALE);
            }
        }
    }
}

rom.onchange = function () {
    var file = this.files[0]
    var reader = new FileReader();
    reader.onload = function(e) {
        chip8.reset();
        chip8.load_program(new Uint8Array(e.target.result));
    }
    reader.readAsArrayBuffer(file);
    updateScreen();
}

document.addEventListener("keydown", event => {
    chip8.key_down(keymap[event.keyCode]);
});

document.addEventListener("keyup", event => {
    chip8.key_up(keymap[event.keyCode]);
});

var paused = true

const renderLoop = () => {
    if (!paused) {
        for (var i = 0; i < SPEED; i++) {
            chip8.execute_next();
        }
        chip8.update_timer();
    }
    updateScreen();
    requestAnimationFrame(renderLoop);
}

const play = () => {
    playBtn.textContent = "||";
    paused = false;
    requestAnimationFrame(renderLoop);
}

const pause = () => {
    playBtn.textContent = "▶";
    paused = true;
    requestAnimationFrame(renderLoop);
}

playBtn.addEventListener("click", event => {
    if (paused) {
        play();
    } else {
        pause();
    }
});

requestAnimationFrame(renderLoop);
