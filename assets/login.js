/**
 * assets/login.js — Interactive Bayer-Dithered Pixel THRALL Canvas Stage,
 * Pointer-Tracked Card Spotlight, and 8-Bit Retro Theme Palette Switcher.
 */
(function () {
  'use strict';

  var ACCENTS = [
    { name: 'Coral', hex: '#ff5555', rgb: [255, 85, 85], hot: '#ff9999', hotRgb: [255, 153, 153] },
    { name: 'Orange', hex: '#ff7700', rgb: [255, 119, 0], hot: '#ffa040', hotRgb: [255, 160, 64] },
    { name: 'Amber', hex: '#ffb000', rgb: [255, 176, 0], hot: '#ffd447', hotRgb: [255, 212, 71] },
    { name: 'Yellow', hex: '#f1fa8c', rgb: [241, 250, 140], hot: '#ffffb0', hotRgb: [255, 255, 176] },
    { name: 'Emerald', hex: '#50fa7b', rgb: [80, 250, 123], hot: '#a2ffbc', hotRgb: [162, 255, 188] },
    { name: 'Cyan', hex: '#00e5ff', rgb: [0, 229, 255], hot: '#80f4ff', hotRgb: [128, 244, 255] },
    { name: 'Blue', hex: '#3b82f6', rgb: [59, 130, 246], hot: '#7cb3ff', hotRgb: [124, 179, 255] },
    { name: 'Violet', hex: '#bd93f9', rgb: [189, 147, 249], hot: '#d9b8ff', hotRgb: [217, 184, 255] }
  ];

  // 70x17 binary pixel grid for "THRALL"
  var THRALL_WORDMARK = [
    '0000000000000000000000000000000000000000000000000000000000000000000000',
    '1111111111100111000011100111111110000001111100000111000000001110000000',
    '1111111111100111000011100111111111000011111110000111000000001110000000',
    '0000111000000111000011100111000011100111000111000111000000001110000000',
    '0000111000000111000011100111000011100111000111000111000000001110000000',
    '0000111000000111000011100111000011100111000111000111000000001110000000',
    '0000111000000111111111100111111111000111000111000111000000001110000000',
    '0000111000000111111111100111111110000111111111000111000000001110000000',
    '0000111000000111111111100111111100000111111111000111000000001110000000',
    '0000111000000111000011100111001110000111000111000111000000001110000000',
    '0000111000000111000011100111000111000111000111000111000000001110000000',
    '0000111000000111000011100111000011100111000111000111000000001110000000',
    '0000111000000111000011100111000011100111000111000111000000001110000000',
    '0000111000000111000011100111000011100111000111000111111111001111111110',
    '0000111000000111000011100111000011100111000111000111111111001111111110',
    '0000000000000000000000000000000000000000000000000000000000000000000000',
    '0000000000000000000000000000000000000000000000000000000000000000000000'
  ];

  window.ThrallWordmark = {
    width: 70,
    height: 17,
    rows: THRALL_WORDMARK
  };

  function applyAccent(a) {
    if (!a) return;
    var r = a.rgb[0], g = a.rgb[1], b = a.rgb[2];
    var hr = a.hotRgb[0], hg = a.hotRgb[1], hb = a.hotRgb[2];
    window.thrallCurrentAccent = a.rgb;
    window.thrallCurrentAccentName = a.name;

    try {
      localStorage.setItem('thrall_accent', a.name.toLowerCase());
    } catch (e) {}

    var root = document.documentElement;
    root.style.setProperty('--phosphor', a.hex);
    root.style.setProperty('--accent', a.hex);
    root.style.setProperty('--accent-rgb', r + ', ' + g + ', ' + b);
    root.style.setProperty('--phosphor-dim', 'rgba(' + r + ', ' + g + ', ' + b + ', 0.12)');
    root.style.setProperty('--phosphor-glow', 'rgba(' + r + ', ' + g + ', ' + b + ', 0.28)');
    root.style.setProperty('--border-accent', 'rgba(' + r + ', ' + g + ', ' + b + ', 0.38)');
    root.style.setProperty('--hot', a.hot);
    root.style.setProperty('--hot-rgb', hr + ', ' + hg + ', ' + hb);
    root.style.setProperty('--spotlight-border', 'rgba(' + r + ', ' + g + ', ' + b + ', 0.52)');
    root.style.setProperty('--spotlight-glow', 'rgba(' + hr + ', ' + hg + ', ' + hb + ', 0.92)');
    root.style.setProperty('--spotlight-inner', 'rgba(' + r + ', ' + g + ', ' + b + ', 0.045)');

    var btns = document.querySelectorAll('[data-accent-name]');
    for (var i = 0; i < btns.length; i++) {
      var btn = btns[i];
      var active = (btn.getAttribute('data-accent-name') || '').toLowerCase() === a.name.toLowerCase();
      btn.classList.toggle('active', active);
      btn.setAttribute('aria-checked', active ? 'true' : 'false');
      btn.setAttribute('tabindex', active ? '0' : '-1');
    }

    if (window.thrallRenderSpotlight) {
      window.thrallRenderSpotlight();
    }
  }

  function initPalette() {
    var saved = 'emerald';
    try {
      saved = localStorage.getItem('thrall_accent') || 'emerald';
    } catch (e) {}
    var initial = null;
    for (var i = 0; i < ACCENTS.length; i++) {
      if (ACCENTS[i].name.toLowerCase() === saved.toLowerCase()) {
        initial = ACCENTS[i];
        break;
      }
    }
    applyAccent(initial || ACCENTS[4]);

    document.addEventListener('click', function (e) {
      var target = e.target;
      if (!target || typeof target.closest !== 'function') return;
      var btn = target.closest('[data-accent-name]');
      if (!btn) return;
      var name = btn.getAttribute('data-accent-name') || '';
      for (var j = 0; j < ACCENTS.length; j++) {
        if (ACCENTS[j].name.toLowerCase() === name.toLowerCase()) {
          applyAccent(ACCENTS[j]);
          break;
        }
      }
    });

    document.addEventListener('keydown', function (e) {
      if (['ArrowLeft', 'ArrowRight', 'ArrowUp', 'ArrowDown'].indexOf(e.key) === -1) return;
      var btn = document.activeElement;
      if (!btn || !btn.matches || !btn.matches('[data-accent-name]')) return;
      var group = btn.closest('[role="radiogroup"]');
      if (!group) return;
      var items = Array.prototype.slice.call(group.querySelectorAll('[data-accent-name]'));
      var idx = items.indexOf(btn);
      if (idx === -1) return;
      e.preventDefault();
      var next = (e.key === 'ArrowRight' || e.key === 'ArrowDown')
        ? (idx + 1) % items.length
        : (idx - 1 + items.length) % items.length;
      items[next].focus();
      items[next].click();
    }, true);
  }

  // Bayer 8x8 dither + Smooth Perlin-style noise plasma + THRALL wordmark canvas
  function initCanvas() {
    var canvas = document.getElementById('spotlight-canvas');
    if (!canvas) return;
    var ctx = canvas.getContext('2d', { alpha: false });
    if (!ctx) return;

    var BAYER = [
      0, 32, 8, 40, 2, 34, 10, 42,
      48, 16, 56, 24, 50, 18, 58, 26,
      12, 44, 4, 36, 14, 46, 6, 38,
      60, 28, 52, 20, 62, 30, 54, 22,
      3, 35, 11, 43, 1, 33, 9, 41,
      51, 19, 59, 27, 49, 17, 57, 25,
      15, 47, 7, 39, 13, 45, 5, 37,
      63, 31, 55, 23, 61, 29, 53, 21
    ];
    var N = 128, smoothAcc = null;
    var noiseTable = (function () {
      var seed = 10407530, buf = new Float32Array(N * N);
      for (var i = 0; i < buf.length; i++) {
        seed = (seed * 1664525 + 1013904223) >>> 0;
        buf[i] = seed / 4294967296;
      }
      for (var p = 0; p < 2; p++) {
        var next = new Float32Array(N * N);
        for (var y = 0; y < N; y++) {
          for (var x = 0; x < N; x++) {
            var sum = 0;
            for (var dy = -1; dy <= 1; dy++) {
              for (var dx = -1; dx <= 1; dx++) {
                sum += buf[((y + dy + N) % N) * N + ((x + dx + N) % N)];
              }
            }
            next[y * N + x] = sum / 9;
          }
        }
        buf = next;
      }
      var min = Infinity, max = -Infinity;
      for (var j = 0; j < buf.length; j++) {
        if (buf[j] < min) min = buf[j];
        if (buf[j] > max) max = buf[j];
      }
      var span = max - min || 1;
      for (var k = 0; k < buf.length; k++) buf[k] = (buf[k] - min) / span;
      return buf;
    })();

    function sampleNoise(x, y) {
      var ix = Math.floor(x), iy = Math.floor(y), fx = x - ix, fy = y - iy;
      var x0 = (ix % N + N) % N, y0 = (iy % N + N) % N, x1 = (x0 + 1) % N, y1 = (y0 + 1) % N;
      var sx = fx * fx * (3 - 2 * fx), sy = fy * fy * (3 - 2 * fy);
      return (noiseTable[y0 * N + x0] * (1 - sx) + noiseTable[y0 * N + x1] * sx) * (1 - sy) +
             (noiseTable[y1 * N + x0] * (1 - sx) + noiseTable[y1 * N + x1] * sx) * sy;
    }

    var TEXT_SEL = 'h1,h2,h3,p,pre,code,label,input,button,a,summary,details,.spotlight-card,.tui-screen-mock,.tenets-marquee-wrap,.footer-dock,.chibi-dock';

    function isOverTextElement(target) {
      if (!target || target === document.body || target === document.documentElement || target === canvas) return false;
      var node = target.nodeType === 3 ? target.parentElement : target;
      if (!node || typeof node.closest !== 'function') return false;
      if (node.id === 'spotlight-canvas' || node.classList.contains('login-stage') || node.classList.contains('wordmark-hero')) return false;
      return Boolean(node.closest(TEXT_SEL));
    }
    window.ThrallIsOverText = isOverTextElement;

    var reducedMotion = window.matchMedia && window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    var dpr = 1, width = 0, height = 0, cellPx = 8, originX = 0, originY = 0, gridCols = 0, gridRows = 0;
    var wmWidth = THRALL_WORDMARK[0].length, wmHeight = THRALL_WORDMARK.length, wmRows = THRALL_WORDMARK;
    var meshCount = Math.max(1, parseInt(canvas.getAttribute('data-mesh-nodes') || '3', 10));
    var rawMini = parseInt(canvas.getAttribute('data-mini-nodes'), 10);
    var miniCount = isNaN(rawMini) ? 0 : Math.max(0, rawMini);
    var rawRtts = (canvas.getAttribute('data-mesh-rtts') || '').split(',');
    var meshRtts = [];
    for (var ri = 0; ri < meshCount; ri++) {
      var rv = parseFloat(rawRtts[ri]);
      meshRtts.push(isNaN(rv) || rv <= 0 ? (ri + 1) * 14 : rv);
    }
    var rawBps = (canvas.getAttribute('data-mesh-bps') || '').split(',');
    var meshBps = [];
    for (var bi = 0; bi < meshCount; bi++) {
      var bv = parseFloat(rawBps[bi]);
      meshBps.push(isNaN(bv) || bv < 0 ? 0 : bv);
    }
    var rawParents = (canvas.getAttribute('data-mini-parents') || '').split(',');
    var miniParents = [];
    for (var pi = 0; pi < miniCount; pi++) {
      var pv = parseInt(rawParents[pi], 10);
      miniParents.push(isNaN(pv) || pv < 0 ? (pi % meshCount) : (pv % meshCount));
    }
    var mouse = { x: -2000, y: -2000, active: false, lastMove: 0 };
    var orb = { x: 0, y: 0, initialized: false };
    var meshOrbs = [], miniOrbs = [];
    var isOverText = false, smoothText = 0, smoothControl = 0;
    var ripples = [], rafId = 0, isVisible = true;
    var spotCards = [];

    function refreshSpotCards() {
      spotCards = Array.prototype.slice.call(document.querySelectorAll('.spotlight-card'));
    }

    function syncCardsToOrbs(orbsList, radiusPx, overInteractive) {
      if (!spotCards.length) refreshSpotCards();
      for (var i = 0; i < spotCards.length; i++) {
        var card = spotCards[i];
        var r = card.getBoundingClientRect();
        var bestDistSq = Infinity, bestX = orb.x / dpr, bestY = orb.y / dpr;
        for (var k = 0; k < orbsList.length; k++) {
          var ox = orbsList[k].x / dpr, oy = orbsList[k].y / dpr;
          var dx = ox < r.left ? r.left - ox : (ox > r.right ? ox - r.right : 0);
          var dy = oy < r.top ? r.top - oy : (oy > r.bottom ? oy - r.bottom : 0);
          var dSq = dx * dx + dy * dy;
          if (dSq < bestDistSq) {
            bestDistSq = dSq;
            bestX = ox;
            bestY = oy;
          }
        }
        if (bestDistSq < radiusPx * radiusPx) {
          card.style.setProperty('--mouse-x', Math.round(bestX - r.left) + 'px');
          card.style.setProperty('--mouse-y', Math.round(bestY - r.top) + 'px');
          card.classList.add('spotlight-active');
          card.classList.toggle('spotlight-over-text', Boolean(overInteractive && bestDistSq === 0));
        } else {
          card.classList.remove('spotlight-active', 'spotlight-over-text');
        }
      }
    }

    function resize() {
      dpr = Math.min(window.devicePixelRatio || 1, 2);
      width = Math.round(window.innerWidth * dpr);
      height = Math.round(window.innerHeight * dpr);
      if (canvas.width !== width || canvas.height !== height) {
        canvas.width = width;
        canvas.height = height;
      }
      var targetW = Math.min(width * 0.86, 840 * dpr);
      cellPx = Math.max(4, Math.floor(targetW / wmWidth));
      originX = Math.round((width - wmWidth * cellPx) / 2);
      var heroEl = document.getElementById('wordmark-anchor');
      if (heroEl) {
        var r = heroEl.getBoundingClientRect();
        originY = Math.round((r.top + r.height * 0.5) * dpr - (wmHeight * cellPx) * 0.5);
      } else {
        originY = Math.round(height * 0.12 - (wmHeight * cellPx) * 0.5);
      }
      gridCols = Math.ceil(width / cellPx) + 1;
      gridRows = Math.ceil(height / cellPx) + 1;
      refreshSpotCards();
      if (reducedMotion) render(performance.now());
    }

    function wakeRender() {
      if (!rafId && !reducedMotion && isVisible) {
        rafId = requestAnimationFrame(render);
      }
    }

    function onPointerMove(e) {
      if (!e || e.clientX < 0 || e.clientY < 0 || e.clientX >= window.innerWidth || e.clientY >= window.innerHeight) {
        deactivate();
        return;
      }
      mouse.x = e.clientX * dpr;
      mouse.y = e.clientY * dpr;
      mouse.active = true;
      mouse.lastMove = performance.now();
      isOverText = isOverTextElement(e.target);
      if (reducedMotion) render(performance.now()); else wakeRender();
    }

    function onPointerDown(e) {
      if (!e) return;
      var px = e.clientX * dpr, py = e.clientY * dpr;
      mouse.x = px;
      mouse.y = py;
      mouse.active = true;
      mouse.lastMove = performance.now();
      isOverText = isOverTextElement(e.target);
      ripples.push({ x: px, y: py, born: performance.now(), maxR: 380 * dpr, dur: 950 });
      if (ripples.length > 6) ripples.shift();
      if (reducedMotion) render(performance.now()); else wakeRender();
    }

    function deactivate() {
      mouse.active = false;
      isOverText = false;
      if (reducedMotion) render(performance.now());
    }

    function render(t) {
      rafId = 0;
      var sec = t * 0.001;
      var patrolX = width * (0.5 + 0.36 * Math.sin(sec * 0.68));
      var patrolY = height * (0.34 + 0.24 * Math.sin(sec * 0.92 + 1.2));
      if (!orb.initialized) {
        orb.x = patrolX;
        orb.y = patrolY;
        orb.initialized = true;
      }

      var idleMs = t - mouse.lastMove;
      var cursorControlled = mouse.active && idleMs < 2400;
      var targetControl = 0;
      var targetText = (isOverText && cursorControlled) ? 1 : 0;
      smoothControl = 0;
      smoothText += (targetText - smoothText) * 0.18;

      orb.x += (patrolX - orb.x) * 0.045;
      orb.y += (patrolY - orb.y) * 0.045;

      var maxOrbitR = 95 * dpr;
      meshOrbs.length = meshCount;
      for (var m = 0; m < meshCount; m++) {
        var rttMs = meshRtts[m] || ((m + 1) * 14);
        var pingDist = Math.min(360, Math.max(92, (46 + Math.sqrt(rttMs) * 16 + m * 22) * 1.5)) * dpr;
        if (pingDist > maxOrbitR) maxOrbitR = pingDist;
        var ang = sec * (0.95 / (1 + 0.06 * Math.sqrt(rttMs))) + (m * 2 * Math.PI) / meshCount;
        var rad = pingDist * (1 + 0.08 * Math.sin(sec * 1.45 + m * 2.1));
        meshOrbs[m] = {
          x: orb.x + Math.cos(ang) * rad,
          y: orb.y + Math.sin(ang) * (rad * 0.78)
        };
      }

      miniOrbs.length = miniCount;
      for (var s = 0; s < miniCount; s++) {
        var pIdx = miniParents[s] !== undefined ? miniParents[s] : (s % meshCount);
        var pOrb = meshOrbs[pIdx] || orb;
        var sAng = sec * (2.25 + (s % 3) * 0.45) * ((s % 2 === 0) ? 1 : -1) + s * 2.09;
        var sRad = (36 + (s % 3) * 12) * (1 + 0.1 * Math.cos(sec * 1.3 + s)) * dpr;
        miniOrbs[s] = {
          x: pOrb.x + Math.cos(sAng) * sRad,
          y: pOrb.y + Math.sin(sAng) * (sRad * 0.85)
        };
      }

      syncCardsToOrbs(meshOrbs, 175, isOverText && cursorControlled);
      ripples = ripples.filter(function (rp) { return (t - rp.born) < rp.dur; });

      ctx.fillStyle = '#06080a';
      ctx.fillRect(0, 0, width, height);

      var textDim = smoothText;
      var reachScale = 1 - 0.58 * textDim;
      var orbReach = 104 * reachScale * dpr;
      var orbReachSq = orbReach * orbReach;
      var miniReach = 34 * reachScale * dpr;
      var miniReachSq = miniReach * miniReach;
      var orbWeight = 0.56 * (1 - 0.55 * textDim);
      var miniWeight = 0.48 * (1 - 0.55 * textDim);
      var ringW = 28 * dpr;

      var activeRipples = ripples.map(function (rp) {
        var progress = (t - rp.born) / rp.dur;
        var curR = rp.maxR * (1 - Math.pow(1 - progress, 3));
        var minR = Math.max(0, curR - ringW), maxR = curR + ringW;
        return { x: rp.x, y: rp.y, r: curR, minRSq: minR * minR, maxRSq: maxR * maxR, amp: Math.pow(1 - progress, 1.8) };
      });

      var glareCut = 1 - 0.55 * textDim;
      var acc = window.thrallCurrentAccent || [80, 250, 123];
      if (!smoothAcc) smoothAcc = [acc[0], acc[1], acc[2]];
      smoothAcc[0] += (acc[0] - smoothAcc[0]) * 0.09;
      smoothAcc[1] += (acc[1] - smoothAcc[1]) * 0.09;
      smoothAcc[2] += (acc[2] - smoothAcc[2]) * 0.09;

      var ar = Math.round(smoothAcc[0]), ag = Math.round(smoothAcc[1]), ab = Math.round(smoothAcc[2]);
      var hr = Math.min(255, Math.round(ar + (255 - ar) * 0.42));
      var hg = Math.min(255, Math.round(ag + (255 - ag) * 0.42));
      var hb = Math.min(255, Math.round(ab + (255 - ab) * 0.42));

      var aFHigh = (0.25 * (1 - 0.28 * textDim)).toFixed(3);
      var aFLow = (0.09 * (1 - 0.28 * textDim)).toFixed(3);
      var colorFieldHigh = 'rgba(' + ar + ',' + ag + ',' + ab + ',' + aFHigh + ')';
      var colorFieldLow = 'rgba(' + ar + ',' + ag + ',' + ab + ',' + aFLow + ')';
      var hotRgb = hr + ',' + hg + ',' + hb;
      var midRgb = ar + ',' + ag + ',' + ab;
      var colorSpotHigh = 'rgba(' + hotRgb + ',' + glareCut.toFixed(2) + ')';
      var colorSpotMid = 'rgba(' + midRgb + ',' + (0.88 * glareCut).toFixed(2) + ')';
      var ditherThreshold = 0.72 + 0.08 * textDim;

      var heroEl = document.getElementById('wordmark-anchor');
      if (heroEl) {
        var rect = heroEl.getBoundingClientRect();
        originY = Math.round((rect.top + rect.height * 0.5) * dpr - (wmHeight * cellPx) * 0.5);
      }

      for (var r = 0; r < gridRows; r++) {
        var py = r * cellPx, wmRow = Math.floor((py - originY) / cellPx);
        var hasWordRow = wmRow >= 0 && wmRow < wmHeight;
        for (var c = 0; c < gridCols; c++) {
          var px = c * cellPx, wmCol = Math.floor((px - originX) / cellPx);
          var inWordBox = hasWordRow && wmCol >= 0 && wmCol < wmWidth;
          var isWord = inWordBox && wmRows[wmRow] && wmRows[wmRow].charAt(wmCol) === '1';

          var cx = px + cellPx * 0.5, cy = py + cellPx * 0.5;
          var spot = 0;
          for (var m = 0; m < meshCount; m++) {
            var dxo = cx - meshOrbs[m].x, dyo = cy - meshOrbs[m].y, dsqO = dxo * dxo + dyo * dyo;
            if (dsqO < orbReachSq) {
              spot += Math.pow(1 - Math.sqrt(dsqO) / orbReach, 2) * orbWeight;
            }
          }
          for (var s = 0; s < miniCount; s++) {
            var dxs = cx - miniOrbs[s].x, dys = cy - miniOrbs[s].y, dsqS = dxs * dxs + dys * dys;
            if (dsqS < miniReachSq) {
              spot += Math.pow(1 - Math.sqrt(dsqS) / miniReach, 2) * miniWeight;
            }
          }
          if (spot > 1.25) spot = 1.25;
          var ripGlow = 0;

          for (var i = 0; i < activeRipples.length; i++) {
            var rp = activeRipples[i], rx = cx - rp.x, ry = cy - rp.y, dsqR = rx * rx + ry * ry;
            if (dsqR >= rp.minRSq && dsqR <= rp.maxRSq) {
              ripGlow = Math.max(ripGlow, (1 - Math.abs(Math.sqrt(dsqR) - rp.r) / ringW) * rp.amp);
            }
          }

          var wordDrawn = false;
          if (isWord) {
            var lit = (0.52 + spot * 1.65 + ripGlow * 1.45) * (1 - 0.38 * textDim);
            if (lit > 0.05) {
              ctx.fillStyle = lit > 1.32
                ? '#ffffff'
                : lit > 0.90
                  ? ('rgb(' + hotRgb + ')')
                  : lit > 0.55
                    ? ('rgb(' + midRgb + ')')
                    : ('rgb(' + Math.round(ar * 0.68) + ',' + Math.round(ag * 0.68) + ',' + Math.round(ab * 0.68) + ')');
              ctx.fillRect(px, py, cellPx - 1, cellPx - 1);
              wordDrawn = true;
            }
          }

          if (!wordDrawn) {
            var nx = c * 0.07 + sec * 0.16, ny = r * 0.07 - sec * 0.11;
            var nVal = 0.6 * sampleNoise(nx, ny) + 0.4 * sampleNoise(nx * 0.5 - sec * 0.08, ny * 0.5 + sec * 0.07);
            var energy = nVal * 0.75 + (0.5 + 0.5 * Math.sin(sec * 1.8 + (r * 37 + c * 11) % 6.283)) * 0.25;
            if (inWordBox) energy *= 0.12;
            var fieldVal = energy * 0.45 + spot * 0.85 + ripGlow * 0.95;
            var dither = BAYER[(r & 7) * 8 + (c & 7)] / 64;
            if (fieldVal > dither * ditherThreshold) {
              ctx.fillStyle = (spot > 0.38 || ripGlow > 0.4)
                ? colorSpotHigh
                : (spot > 0.14 || ripGlow > 0.15)
                  ? colorSpotMid
                  : fieldVal > 0.56
                    ? colorFieldHigh
                    : colorFieldLow;
              ctx.fillRect(px, py, cellPx - 1, cellPx - 1);
            }
          }
        }
      }

      function drawPacketStream(x1, y1, x2, y2, rateBps, seed) {
        var dx = x2 - x1, dy = y2 - y1;
        var dist = Math.hypot(dx, dy);
        var steps = Math.max(4, Math.floor(dist / (cellPx * 2.2)));
        ctx.fillStyle = colorFieldHigh;
        for (var k = 1; k < steps; k += 2) {
          var f = k / steps;
          var lx = Math.round((x1 + dx * f) / cellPx) * cellPx;
          var ly = Math.round((y1 + dy * f) / cellPx) * cellPx;
          ctx.fillRect(lx, ly, cellPx - 1, cellPx - 1);
        }
        var pktCnt = rateBps > 100000 ? 4 : (rateBps > 1000 ? 3 : 2);
        var speed = 0.55 + Math.min(1.6, Math.log10((rateBps || 0) + 10) * 0.22);
        for (var p = 0; p < pktCnt; p++) {
          var prog = (sec * speed + seed * 0.29 + p / pktCnt) % 1;
          var px1 = Math.round((x1 + dx * prog) / cellPx) * cellPx;
          var py1 = Math.round((y1 + dy * prog) / cellPx) * cellPx;
          ctx.fillStyle = '#ffffff';
          ctx.fillRect(px1, py1, cellPx - 1, cellPx - 1);
          var tail = Math.max(0, prog - 0.045);
          var tx1 = Math.round((x1 + dx * tail) / cellPx) * cellPx;
          var ty1 = Math.round((y1 + dy * tail) / cellPx) * cellPx;
          ctx.fillStyle = colorSpotHigh;
          ctx.fillRect(tx1, ty1, cellPx - 1, cellPx - 1);
        }
      }

      if (meshCount > 1) {
        for (var a = 0; a < meshCount; a++) {
          for (var b = a + 1; b < meshCount; b++) {
            var pairBps = (meshBps[a] || 0) + (meshBps[b] || 0);
            drawPacketStream(meshOrbs[a].x, meshOrbs[a].y, meshOrbs[b].x, meshOrbs[b].y, pairBps, a * 3 + b);
            if (pairBps > 500) {
              drawPacketStream(meshOrbs[b].x, meshOrbs[b].y, meshOrbs[a].x, meshOrbs[a].y, pairBps, b * 5 + a);
            }
          }
        }
      }
      for (var s = 0; s < miniCount; s++) {
        var pIdx2 = miniParents[s] !== undefined ? miniParents[s] : (s % meshCount);
        var pOrb2 = meshOrbs[pIdx2] || orb;
        drawPacketStream(pOrb2.x, pOrb2.y, miniOrbs[s].x, miniOrbs[s].y, meshBps[pIdx2] || 200, s + 7);
      }

      for (var m = 0; m < meshCount; m++) {
        ctx.fillStyle = colorSpotHigh;
        var mx = Math.round(meshOrbs[m].x / cellPx) * cellPx;
        var my = Math.round(meshOrbs[m].y / cellPx) * cellPx;
        ctx.fillRect(mx - cellPx, my, cellPx - 1, cellPx - 1);
        ctx.fillRect(mx + cellPx, my, cellPx - 1, cellPx - 1);
        ctx.fillRect(mx, my - cellPx, cellPx - 1, cellPx - 1);
        ctx.fillRect(mx, my + cellPx, cellPx - 1, cellPx - 1);
        ctx.fillStyle = '#ffffff';
        ctx.fillRect(mx, my, cellPx - 1, cellPx - 1);
      }
      for (var s = 0; s < miniCount; s++) {
        var sx = Math.round(miniOrbs[s].x / cellPx) * cellPx;
        var sy = Math.round(miniOrbs[s].y / cellPx) * cellPx;
        ctx.fillStyle = '#ffffff';
        ctx.fillRect(sx, sy, cellPx - 1, cellPx - 1);
      }

      if (!reducedMotion && isVisible) {
        rafId = requestAnimationFrame(render);
      }
    }

    window.thrallRenderSpotlight = function () {
      wakeRender();
      if (reducedMotion) render(performance.now());
    };

    resize();
    render(performance.now());
    var scroller = document.getElementById('middle-scroll');
    window.addEventListener('resize', function () { resize(); wakeRender(); }, { passive: true });
    window.addEventListener('scroll', function () { wakeRender(); }, { passive: true });
    if (scroller) scroller.addEventListener('scroll', function () { wakeRender(); }, { passive: true });
    window.addEventListener('pointermove', onPointerMove, { passive: true });
    window.addEventListener('pointerdown', onPointerDown, { passive: true });
    window.addEventListener('pointerup', function (e) { if (e && e.pointerType === 'touch') deactivate(); }, { passive: true });
    ['pointercancel', 'blur'].forEach(function (ev) { window.addEventListener(ev, deactivate, { passive: true }); });
    ['pointerleave', 'mouseleave'].forEach(function (ev) { document.addEventListener(ev, deactivate, { passive: true }); });
    document.addEventListener('visibilitychange', function () {
      isVisible = !document.hidden;
      if (isVisible) wakeRender();
      else { deactivate(); smoothControl = 0; if (rafId) { cancelAnimationFrame(rafId); rafId = 0; } }
    });
  }

  function initSectionNav() {
    var upBtn = document.getElementById('nav-arrow-up');
    var downBtn = document.getElementById('nav-arrow-down');
    var scroller = document.getElementById('middle-scroll');
    if (!upBtn && !downBtn) return;

    var ids = ['sec-thrall', 'sec-ticker', 'sec-telemetry', 'sec-chibi'];
    var activeIdx = 0;
    var programmatic = false;
    var progTimer = 0;

    function getScrollY() {
      return scroller ? scroller.scrollTop : (window.scrollY || window.pageYOffset || 0);
    }

    function getViewportH() {
      return scroller ? scroller.clientHeight : (window.innerHeight || document.documentElement.clientHeight || 0);
    }

    function getDocH() {
      return scroller ? scroller.scrollHeight : (document.documentElement.scrollHeight || 0);
    }

    function getSections() {
      var list = [];
      for (var i = 0; i < ids.length; i++) {
        var el = document.getElementById(ids[i]);
        if (el) list.push(el);
      }
      return list;
    }

    function computeScrollIdx(secs) {
      if (!secs.length) return 0;
      var sy = getScrollY();
      var winH = getViewportH();
      var docH = getDocH();
      if (sy <= 16) return 0;
      if (docH > winH && sy + winH >= docH - 12) return secs.length - 1;
      var best = 0;
      for (var i = 0; i < secs.length; i++) {
        if (secs[i].getBoundingClientRect().top <= 160) best = i;
      }
      return best;
    }

    function updateButtons(secs) {
      var sy = getScrollY();
      var winH = getViewportH();
      var docH = getDocH();
      var atTop = activeIdx <= 0 && (programmatic || sy <= 16);
      var atBottom = activeIdx >= secs.length - 1 || (!programmatic && docH > winH && sy + winH >= docH - 12);
      if (upBtn) {
        upBtn.disabled = atTop;
        upBtn.classList.toggle('disabled', atTop);
        upBtn.setAttribute('aria-disabled', atTop ? 'true' : 'false');
      }
      if (downBtn) {
        downBtn.disabled = atBottom;
        downBtn.classList.toggle('disabled', atBottom);
        downBtn.setAttribute('aria-disabled', atBottom ? 'true' : 'false');
      }
    }

    function syncFromScroll() {
      if (programmatic) return;
      var secs = getSections();
      activeIdx = computeScrollIdx(secs);
      updateButtons(secs);
    }

    function targetScrollY(secs, idx) {
      if (idx <= 0 || !secs[idx]) return 0;
      var sy = getScrollY();
      var winH = getViewportH();
      var docH = getDocH();
      var maxScroll = Math.max(0, docH - winH);
      var baseTop = scroller ? scroller.getBoundingClientRect().top : 76;
      var raw = Math.round(secs[idx].getBoundingClientRect().top - baseTop + sy - 16);
      return Math.max(0, Math.min(maxScroll, raw));
    }

    function goTo(targetIdx, dir) {
      var secs = getSections();
      if (!secs.length) return;
      var sy = getScrollY();
      var winH = getViewportH();
      var docH = getDocH();
      var maxScroll = Math.max(0, docH - winH);
      var clamped = Math.max(0, Math.min(secs.length - 1, targetIdx));
      if (dir < 0) {
        while (clamped > 0 && Math.abs(targetScrollY(secs, clamped) - sy) < 8) {
          clamped -= 1;
        }
      } else if (dir > 0) {
        while (clamped < secs.length - 1 && Math.abs(targetScrollY(secs, clamped) - sy) < 8) {
          clamped += 1;
        }
      }
      var destY = targetScrollY(secs, clamped);
      if (dir > 0 && maxScroll > 0 && destY >= maxScroll - 8) {
        clamped = secs.length - 1;
      } else if (dir < 0 && destY <= 8) {
        clamped = 0;
      }
      activeIdx = clamped;
      programmatic = true;
      if (progTimer) clearTimeout(progTimer);
      progTimer = setTimeout(function () {
        programmatic = false;
        updateButtons(getSections());
      }, 650);
      if (scroller) {
        scroller.scrollTo({ top: destY, behavior: 'smooth' });
      } else {
        window.scrollTo({ top: destY, behavior: 'smooth' });
      }
      updateButtons(secs);
    }

    if (upBtn) {
      upBtn.addEventListener('click', function () {
        goTo(activeIdx - 1, -1);
      });
    }
    if (downBtn) {
      downBtn.addEventListener('click', function () {
        goTo(activeIdx + 1, 1);
      });
    }

    document.addEventListener('keydown', function (e) {
      if (e.key !== 'ArrowUp' && e.key !== 'ArrowDown') return;
      if (e.defaultPrevented) return;
      var act = document.activeElement;
      var tag = act && act.tagName ? act.tagName.toUpperCase() : '';
      if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT' || (act && act.isContentEditable)) return;
      if (act && act.matches && act.matches('[data-accent-name]')) return;
      e.preventDefault();
      if (e.key === 'ArrowDown') goTo(activeIdx + 1, 1);
      else goTo(activeIdx - 1, -1);
    });

    if (scroller) scroller.addEventListener('scroll', syncFromScroll, { passive: true });
    window.addEventListener('scroll', syncFromScroll, { passive: true });
    window.addEventListener('resize', syncFromScroll, { passive: true });
    ['wheel', 'touchstart'].forEach(function (ev) {
      window.addEventListener(ev, function () {
        if (programmatic) {
          programmatic = false;
          if (progTimer) { clearTimeout(progTimer); progTimer = 0; }
        }
      }, { passive: true });
    });

    syncFromScroll();
  }

  function initChibiHints() {
    var tipsByTab = {
      0: [
        { title: 'HANDSHAKE', body: 'Sign in above, then go to Tab 2: add the Gateway ID in localhost:8384 and paste your 56-char ID into THRALL.' },
        { title: 'PRIVACY', body: 'Workstations only connect to regional Thrall Nodes over TLS 1.3 BEP—never directly to other users\' devices.' },
        { title: 'VAULTS', body: 'THRALL never syncs ~/.config or dotfiles. Only explicit 1A Personal and 1B Shared folders replicate.' },
        { title: 'PALETTE', body: 'Click anywhere for a shockwave ripple, or pick one of the 8-bit color swatches in the footer.' }
      ],
      1: [
        { title: 'ACCEPT SHARES', body: 'When a folder card says PENDING, open http://127.0.0.1:8384 on that workstation to click Accept.' },
        { title: '1A vs. 1B', body: '1A Personal Folders sync only to your devices; 1B Shared Folders are multi-user team vaults.' },
        { title: 'DEVICE BADGES', body: 'Each folder card shows \u2713 accepted vs. \u23F3 unaccepted per workstation so you know which machine needs to accept.' },
        { title: 'PUBLIC LINKS', body: 'In the Explorer below, click "Share: OFF" on any file or subfolder to generate a read-only /s/<token> link.' },
        { title: 'FOLDER IDs', body: 'Every folder uses a native 5+5 char Syncthing Folder ID (xxxxx-xxxxx) and rejects dotfiles/symlinks.' }
      ],
      2: [
        { title: 'STEP 1', body: 'Open http://127.0.0.1:8384, click + Add Remote Device, paste the Gateway ID, and check [x] Introducer & [x] Auto Accept.' },
        { title: 'STEP 2', body: 'In localhost:8384 click Actions -> Show ID, copy your 56-character Device ID, and paste it into Step 2.' },
        { title: 'AUTO-ACCEPT', body: '[x] Introducer peers you with all regional nodes; [x] Auto Accept mounts your 1A & 1B folders automatically.' },
        { title: 'PASTE FORMAT', body: 'Spaces or hyphens don\'t matter when pasting your ID—THRALL normalizes and verifies all 4 Luhn check digits.' }
      ],
      3: [
        { title: 'AUTO-NAMES', body: 'Just paste your 56-char Device ID—THRALL automatically learns your workstation\'s hostname when it connects.' },
        { title: 'MULTI-DEVICE', body: 'Enroll all your workstations here; every 1A Personal and 1B Shared folder pushes to each enrolled device.' },
        { title: 'FAST REVOKE', body: 'Clicking Revoke purges that device and its folder grants from all regional Thrall Nodes in under a second.' },
        { title: 'RE-ENROLL', body: 'Pasting an already-enrolled Device ID is safe and refreshes its folder grants across the mesh.' }
      ],
      4: [
        { title: 'TEAM ACLs', body: 'Check or uncheck users in the aligned columns and click Apply ACL to grant or revoke Shared Folders.' },
        { title: 'CHECKMARKS', body: 'A \u2713 next to a username means all of that user\'s enrolled workstations have accepted the shared folder.' },
        { title: 'ISOLATION', body: 'Shared folders live at /shared/<Folder> outside individual user home directories.' },
        { title: 'REVOCATION', body: 'Unchecking a user and clicking Apply ACL immediately removes that share from all of their devices.' }
      ],
      5: [
        { title: 'BACKBONE', body: 'Regional Thrall Nodes form a full mesh over port 22000 while serving Client Devices as hub-and-spoke.' },
        { title: 'FAST SSH', body: 'THRALL manages regional nodes over persistent OpenSSH ControlMaster sockets for sub-second updates.' },
        { title: 'NODE CONTROL', body: 'Click Disconnect to isolate a regional server from the mesh, or Reconcile for a full fleet refresh.' },
        { title: 'ADD NODE', body: 'Enter a node name and Tailscale/mesh IP to peer a new regional server into the mesh.' }
      ]
    };

    var heroTitle = document.getElementById('chibi-tip-title');
    var heroBody = document.getElementById('chibi-tip-body');
    var heroNext = document.getElementById('chibi-next-btn');
    var heroImg = document.getElementById('chibi-hero-img');

    var dock = document.getElementById('chibi-dock');
    var dockTitle = document.getElementById('chibi-dock-title');
    var dockBody = document.getElementById('chibi-dock-body');
    var dockNext = document.getElementById('chibi-dock-next');
    var dockBtn = document.getElementById('chibi-mascot-btn');

    if (!heroTitle && !dockTitle) return;

    var curTab = dock ? parseInt(dock.getAttribute('data-tab') || '0', 10) : 0;
    var pool = tipsByTab[curTab] || tipsByTab[0];
    var idx = Math.floor(Math.random() * pool.length);

    function renderTip(i) {
      var t = pool[i % pool.length];
      var num = ((i % pool.length) + 1) + '/' + pool.length;
      if (heroTitle && heroBody) {
        heroTitle.textContent = '\uD83D\uDC41 CHIBI THRALL // TIP ' + num + ' — ' + t.title;
        heroBody.textContent = t.body;
      }
      if (dockTitle && dockBody) {
        dockTitle.textContent = '[' + t.title + ' ' + num + ']';
        dockBody.textContent = t.body;
        dockBody.title = t.body;
      }
    }

    function advanceTip() {
      if (pool.length > 1) {
        var step = 1 + Math.floor(Math.random() * (pool.length - 1));
        idx = (idx + step) % pool.length;
      }
      renderTip(idx);
    }

    renderTip(idx);
    if (heroNext) heroNext.addEventListener('click', advanceTip);
    if (heroImg) heroImg.addEventListener('click', advanceTip);
    if (dockNext) dockNext.addEventListener('click', advanceTip);
    if (dockBtn) dockBtn.addEventListener('click', advanceTip);
    setInterval(advanceTip, 14000);
  }

  function start() {
    initPalette();
    initCanvas();
    initSectionNav();
    initChibiHints();
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', start);
  } else {
    start();
  }
})();
