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

    if (window.thrallRenderHeaderWordmark) {
      window.thrallRenderHeaderWordmark();
    }
    if (window.thrallRenderSpotlight) {
      window.thrallRenderSpotlight();
    }
  }

  function renderHeaderWordmark() {
    var hCanvas = document.getElementById('header-wordmark');
    if (!hCanvas || !hCanvas.getContext) return;
    var hCtx = hCanvas.getContext('2d');
    if (!hCtx) return;
    var cols = THRALL_WORDMARK[0].length, rows = THRALL_WORDMARK.length;
    var scale = 2;
    hCanvas.width = cols * scale;
    hCanvas.height = rows * scale;
    hCtx.clearRect(0, 0, hCanvas.width, hCanvas.height);
    var acc = window.thrallCurrentAccent || [80, 250, 123];
    var ar = acc[0], ag = acc[1], ab = acc[2];
    var hr = Math.min(255, Math.round(ar + (255 - ar) * 0.48));
    var hg = Math.min(255, Math.round(ag + (255 - ag) * 0.48));
    var hb = Math.min(255, Math.round(ab + (255 - ab) * 0.48));
    for (var r = 0; r < rows; r++) {
      var rowStr = THRALL_WORDMARK[r];
      for (var c = 0; c < cols; c++) {
        if (rowStr.charAt(c) === '1') {
          hCtx.fillStyle = r < 5 ? '#ffffff' : (r < 11 ? ('rgb(' + hr + ',' + hg + ',' + hb + ')') : ('rgb(' + ar + ',' + ag + ',' + ab + ')'));
          hCtx.fillRect(c * scale, r * scale, scale - 0.35, scale - 0.35);
        }
      }
    }
  }
  window.thrallRenderHeaderWordmark = renderHeaderWordmark;

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
    var meshBps = [], totalBps = 0;
    for (var bi = 0; bi < meshCount; bi++) {
      var bv = parseFloat(rawBps[bi]);
      var cleanB = isNaN(bv) || bv < 0 ? 0 : bv;
      meshBps.push(cleanB);
      totalBps += cleanB;
    }
    var rawMasses = (canvas.getAttribute('data-mesh-masses') || '').split(',');
    var meshMasses = [];
    for (var mi = 0; mi < meshCount; mi++) {
      var mv = parseFloat(rawMasses[mi]);
      meshMasses.push(isNaN(mv) || mv <= 0 ? (mi + 1) * 1024 : mv);
    }
    var rawBearings = (canvas.getAttribute('data-mesh-bearings') || '').split(',');
    var meshBearings = [];
    for (var bri = 0; bri < meshCount; bri++) {
      var brv = parseFloat(rawBearings[bri]);
      var deg = isNaN(brv) ? (bri * 360 / meshCount) : brv;
      meshBearings.push((deg * Math.PI) / 180);
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
      originX = -99999;
      originY = -99999;
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
      var throughputBoost = Math.min(2.4, Math.log10((totalBps || 0) + 10) * 0.32);
      var spinRate = 0.22 + throughputBoost * 0.38;
      var baryDx = 0, baryDy = 0, weightSum = 0;
      for (var bm = 0; bm < meshCount; bm++) {
        var mw = Math.log10((meshMasses[bm] || 1024) + 10);
        var bAng = meshBearings[bm] + sec * spinRate;
        baryDx += Math.cos(bAng) * mw;
        baryDy += Math.sin(bAng) * mw;
        weightSum += mw;
      }
      if (weightSum > 0) {
        baryDx /= weightSum;
        baryDy /= weightSum;
      }
      var patrolX = width * (0.5 + 0.18 * Math.sin(sec * 0.42) + 0.08 * baryDx);
      var patrolY = height * (0.34 + 0.14 * Math.sin(sec * 0.58 + 1.2) + 0.07 * baryDy);
      if (!orb.initialized) {
        orb.x = patrolX;
        orb.y = patrolY;
        orb.initialized = true;
      }

      var idleMs = t - mouse.lastMove;
      var cursorControlled = mouse.active && idleMs < 2400;
      smoothControl = 0;
      smoothText = 0;

      orb.x += (patrolX - orb.x) * 0.045;
      orb.y += (patrolY - orb.y) * 0.045;

      var maxOrbitR = 290 * dpr;
      meshOrbs.length = meshCount;
      for (var m = 0; m < meshCount; m++) {
        var rttMs = meshRtts[m] || ((m + 1) * 14);
        var nodeBps = meshBps[m] || 0;
        var gravPull = Math.min(0.14, Math.log10(nodeBps + 1) * 0.025);
        var pingDist = Math.min(780, Math.max(290, (140 + Math.sqrt(rttMs) * 34 + m * 55) * 2.4 * (1 - gravPull))) * dpr;
        if (pingDist > maxOrbitR) maxOrbitR = pingDist;
        var evenAngle = (m * 2 * Math.PI) / meshCount;
        var rawBrg = meshBearings[m] !== undefined ? meshBearings[m] : evenAngle;
        var baseBearing = evenAngle + ((rawBrg % 0.42) - 0.21);
        var nodeSpin = spinRate / (1 + 0.05 * Math.sqrt(rttMs));
        var ang = baseBearing + sec * nodeSpin + 0.08 * Math.sin(sec * 0.9 + m * 1.7);
        var rad = pingDist * (1 + 0.05 * Math.sin(sec * 1.35 + m * 2.1));
        var jitter = rttMs > 45 ? Math.sin(sec * 14 + m) * 2.5 * dpr : 0;
        meshOrbs[m] = {
          x: orb.x + Math.cos(ang) * rad + jitter,
          y: orb.y + Math.sin(ang) * (rad * 0.72) - jitter
        };
      }

      miniOrbs.length = miniCount;
      for (var s = 0; s < miniCount; s++) {
        var pIdx = miniParents[s] !== undefined ? miniParents[s] : (s % meshCount);
        var pOrb = meshOrbs[pIdx] || orb;
        var sAng = sec * (1.45 + (s % 3) * 0.45) * ((s % 2 === 0) ? 1 : -1) + s * 2.09;
        var sRad = (155 + (s % 3) * 32) * (1 + 0.06 * Math.cos(sec * 1.3 + s)) * dpr;
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
      var orbReach = 108 * reachScale * dpr;
      var orbReachSq = orbReach * orbReach;
      var orbWeight = 0.62 * (1 - 0.55 * textDim);
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

      for (var r = 0; r < gridRows; r++) {
        var py = r * cellPx;
        for (var c = 0; c < gridCols; c++) {
          var px = c * cellPx;
          var cx = px + cellPx * 0.5, cy = py + cellPx * 0.5;
          var spot = 0;
          for (var m = 0; m < meshCount; m++) {
            var dxo = cx - meshOrbs[m].x, dyo = cy - meshOrbs[m].y, dsqO = dxo * dxo + dyo * dyo;
            if (dsqO < orbReachSq) {
              spot += Math.pow(1 - Math.sqrt(dsqO) / orbReach, 2) * orbWeight;
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

          var nx = c * 0.07 + sec * 0.16, ny = r * 0.07 - sec * 0.11;
          var nVal = 0.6 * sampleNoise(nx, ny) + 0.4 * sampleNoise(nx * 0.5 - sec * 0.08, ny * 0.5 + sec * 0.07);
          var energy = nVal * 0.75 + (0.5 + 0.5 * Math.sin(sec * 1.8 + (r * 37 + c * 11) % 6.283)) * 0.25;
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

      function drawPacketStream(x1, y1, x2, y2, rateBps, seed) {
        var dx = x2 - x1, dy = y2 - y1;
        var dist = Math.hypot(dx, dy);
        var steps = Math.max(4, Math.floor(dist / (cellPx * 3.2)));
        ctx.fillStyle = colorFieldLow;
        for (var k = 1; k < steps; k += 2) {
          var f = k / steps;
          var lx = Math.round((x1 + dx * f) / cellPx) * cellPx;
          var ly = Math.round((y1 + dy * f) / cellPx) * cellPx;
          ctx.fillRect(lx, ly, cellPx - 1, cellPx - 1);
        }
        var pktCnt = rateBps > 100000 ? 3 : (rateBps > 1000 ? 2 : 1);
        var speed = 0.45 + Math.min(1.4, Math.log10((rateBps || 0) + 10) * 0.2);
        for (var p = 0; p < pktCnt; p++) {
          var prog = 0.08 + ((sec * speed + seed * 0.29 + p / pktCnt) % 1) * 0.84;
          var px1 = Math.round((x1 + dx * prog) / cellPx) * cellPx;
          var py1 = Math.round((y1 + dy * prog) / cellPx) * cellPx;
          ctx.fillStyle = colorSpotMid;
          ctx.fillRect(px1, py1, cellPx - 1, cellPx - 1);
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
        var mx = Math.round(meshOrbs[m].x / cellPx) * cellPx;
        var my = Math.round(meshOrbs[m].y / cellPx) * cellPx;
        var massLog = Math.min(3, Math.max(1, Math.floor(Math.log10((meshMasses[m] || 1024) + 10) / 2.2)));
        ctx.save();
        ctx.strokeStyle = colorSpotHigh;
        ctx.lineWidth = Math.max(2, dpr * 1.8);
        var mPad = cellPx * (1.8 + massLog * 0.45);
        ctx.strokeRect(mx - mPad, my - mPad, cellPx + mPad * 2 - 1, cellPx + mPad * 2 - 1);
        ctx.fillStyle = colorSpotMid;
        for (var dyM = -2; dyM <= 2; dyM++) {
          for (var dxM = -2; dxM <= 2; dxM++) {
            if (Math.abs(dxM) + Math.abs(dyM) <= 2 + (massLog > 1 ? 1 : 0)) {
              ctx.fillRect(mx + dxM * cellPx, my + dyM * cellPx, cellPx - 1, cellPx - 1);
            }
          }
        }
        ctx.fillStyle = colorSpotHigh;
        for (var dyC = -1; dyC <= 1; dyC++) {
          for (var dxC = -1; dxC <= 1; dxC++) {
            ctx.fillRect(mx + dxC * cellPx, my + dyC * cellPx, cellPx - 1, cellPx - 1);
          }
        }
        ctx.fillStyle = '#ffffff';
        ctx.fillRect(mx, my, cellPx - 1, cellPx - 1);
        ctx.font = '700 ' + Math.round(11 * dpr) + 'px ui-monospace, monospace';
        ctx.fillStyle = colorSpotHigh;
        ctx.fillText('HUB-0' + (m + 1), mx + mPad + 6 * dpr, my + 4 * dpr);
        ctx.restore();
      }
      for (var s = 0; s < miniCount; s++) {
        var sx = Math.round(miniOrbs[s].x / cellPx) * cellPx;
        var sy = Math.round(miniOrbs[s].y / cellPx) * cellPx;
        ctx.save();
        ctx.strokeStyle = 'rgba(0, 229, 255, 0.88)';
        ctx.lineWidth = Math.max(1.2, dpr * 1.1);
        var pad = cellPx * 0.85;
        ctx.strokeRect(sx - pad, sy - pad, cellPx + pad * 2 - 1, cellPx + pad * 2 - 1);
        ctx.fillStyle = '#00e5ff';
        ctx.fillRect(sx, sy, cellPx - 1, cellPx - 1);
        ctx.fillStyle = '#ffffff';
        var inner = Math.max(2, Math.floor(cellPx * 0.45));
        var off = Math.floor((cellPx - 1 - inner) * 0.5);
        ctx.fillRect(sx + off, sy + off, inner, inner);
        ctx.font = '700 ' + Math.round(9.5 * dpr) + 'px ui-monospace, monospace';
        ctx.fillStyle = '#00e5ff';
        ctx.fillText('SPOKE-0' + (s + 1), sx + pad + 4 * dpr, sy + 3 * dpr);
        ctx.restore();
      }

      if (!reducedMotion && isVisible) {
        rafId = requestAnimationFrame(render);
      }
    }

    window.thrallRenderSpotlight = function () {
      wakeRender();
      if (reducedMotion) render(performance.now());
    };

    window.thrallApplyLiveTelemetry = function (d) {
      if (!d || typeof d !== 'object') return;
      if (typeof d.hubs_total === 'number' && d.hubs_total >= 1) meshCount = d.hubs_total;
      if (typeof d.spokes === 'number' && d.spokes >= 0) miniCount = d.spokes;
      if (Array.isArray(d.rtts)) {
        for (var i = 0; i < meshCount; i++) {
          var rv = parseFloat(d.rtts[i]);
          meshRtts[i] = isNaN(rv) || rv <= 0 ? (i + 1) * 14 : rv;
        }
      }
      if (Array.isArray(d.bps)) {
        totalBps = 0;
        for (var j = 0; j < meshCount; j++) {
          var bv = parseFloat(d.bps[j]);
          var cb = isNaN(bv) || bv < 0 ? 0 : bv;
          meshBps[j] = cb;
          totalBps += cb;
        }
      }
      if (Array.isArray(d.masses)) {
        for (var k = 0; k < meshCount; k++) {
          var mv = parseFloat(d.masses[k]);
          meshMasses[k] = isNaN(mv) || mv <= 0 ? (k + 1) * 1024 : mv;
        }
      }
      if (Array.isArray(d.bearings)) {
        for (var b = 0; b < meshCount; b++) {
          var brv = parseFloat(d.bearings[b]);
          var deg = isNaN(brv) ? (b * 360 / meshCount) : brv;
          meshBearings[b] = (deg * Math.PI) / 180;
        }
      }
      if (Array.isArray(d.parents)) {
        for (var p = 0; p < miniCount; p++) {
          var pv = parseInt(d.parents[p], 10);
          miniParents[p] = isNaN(pv) || pv < 0 ? (p % meshCount) : (pv % meshCount);
        }
      }
      wakeRender();
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

    var ids = ['sec-thrall', 'sec-telemetry'];
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

  function padRight(str, len) {
    var s = String(str);
    while (s.length < len) s += ' ';
    return s;
  }

  function initTelemetryPolling() {
    function poll() {
      if (document.hidden || typeof window.fetch !== 'function') return;
      fetch('/api/telemetry', { cache: 'no-store', credentials: 'same-origin' })
        .then(function (r) { return r.ok ? r.json() : null; })
        .then(function (d) {
          if (!d) return;
          if (window.thrallApplyLiveTelemetry) window.thrallApplyLiveTelemetry(d);
          var elHubs = document.getElementById('tui-hubs');
          if (elHubs) elHubs.textContent = padRight(d.hubs_online + '/' + d.hubs_total, 8);
          var elLinks = document.getElementById('tui-links');
          if (elLinks) elLinks.textContent = padRight(d.active_links, 6);
          var elUp = document.getElementById('tui-uptime');
          if (elUp) elUp.textContent = padRight(d.uptime_h + 'h', 11);
          var elRx = document.getElementById('tui-rx');
          if (elRx && d.rx) elRx.textContent = '\u25B8 ' + padRight(d.rx, 16);
          var elTx = document.getElementById('tui-tx');
          if (elTx && d.tx) elTx.textContent = '\u25B8 ' + padRight(d.tx, 16);
          var elSum = document.getElementById('footer-fleet-summary');
          if (elSum) {
            var hw = d.hubs_total === 1 ? 'HUB' : 'HUBS';
            var sw = d.spokes === 1 ? 'SPOKE' : 'SPOKES';
            elSum.textContent = d.hubs_total + ' MESH ' + hw + ' \u2022 ' + d.spokes + ' ENROLLED ' + sw;
          }
        })
        .catch(function () {});
    }
    setInterval(poll, 15000);
  }

  function start() {
    initPalette();
    renderHeaderWordmark();
    initCanvas();
    initSectionNav();
    initChibiHints();
    initTelemetryPolling();
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', start);
  } else {
    start();
  }
})();
