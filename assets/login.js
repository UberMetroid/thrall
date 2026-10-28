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

  // Administrator-set appearance, delivered as body data attributes because the
  // portal's Content-Security-Policy forbids inline script.
  function uiAttr(name) {
    if (!document.body) return '';
    return document.body.getAttribute('data-' + name) || '';
  }

  function applyAccent(a, persist) {
    if (!a) return;
    var r = a.rgb[0], g = a.rgb[1], b = a.rgb[2];
    var hr = a.hotRgb[0], hg = a.hotRgb[1], hb = a.hotRgb[2];
    window.thrallCurrentAccent = a.rgb;
    window.thrallCurrentAccentName = a.name;

    if (persist !== false) {
      try {
        localStorage.setItem('thrall_accent', a.name.toLowerCase());
      } catch (e) {}
    }

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
    if (window.thrallRenderFooterGraph) {
      window.thrallRenderFooterGraph();
    }
    if (window.thrallRenderFooterWidgets) {
      window.thrallRenderFooterWidgets();
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
    // A locked palette is the administrator's theme for everyone: the stored
    // browser choice is ignored and never written.
    var locked = uiAttr('palette') === '0';
    var fallback = (uiAttr('accent-default') || 'emerald').toLowerCase();
    var saved = fallback;
    if (!locked) {
      try {
        saved = (localStorage.getItem('thrall_accent') || fallback).toLowerCase();
      } catch (e) {}
    }
    var initial = null;
    for (var i = 0; i < ACCENTS.length; i++) {
      if (ACCENTS[i].name.toLowerCase() === saved.toLowerCase()) {
        initial = ACCENTS[i];
        break;
      }
    }
    applyAccent(initial || ACCENTS[4], !locked);

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

  // Background image: the administrator picks the mode per surface. "off"
  // removes the image entirely, "pin" fixes one image, "random" keeps the
  // per-visit rotation.
  var bgMode = uiAttr('bg') || 'random';
  var bgPinned = parseInt(uiAttr('bg-index') || '1', 10);
  var bgIdx = 0;
  if (bgMode === 'pin') {
    bgIdx = ((Math.max(1, Math.min(5, bgPinned)) - 1) % 5) + 1;
  } else if (bgMode !== 'off') {
    try {
      var prevBg = parseInt(sessionStorage.getItem('thrall_bg_idx') || '0', 10);
      if (prevBg >= 1 && prevBg <= 5) {
        bgIdx = ((prevBg - 1 + 1 + Math.floor(Math.random() * 4)) % 5) + 1;
      } else {
        bgIdx = 1 + Math.floor(Math.random() * 5);
      }
      sessionStorage.setItem('thrall_bg_idx', String(bgIdx));
    } catch (e) {
      bgIdx = 1 + Math.floor(Math.random() * 5);
    }
  }
  window.thrallBgIdx = bgIdx;

  function applyBackground(idx) {
    if (!document.body) return;
    if (!idx) {
      document.body.style.backgroundImage = '';
      return;
    }
    document.body.style.backgroundImage = "linear-gradient(rgba(7, 8, 9, 0.86), rgba(7, 8, 9, 0.92)), url('/bg-" + idx + ".jpg')";
  }
  applyBackground(bgIdx);

  // Bayer 8x8 dither + Smooth Perlin-style noise plasma + THRALL wordmark canvas
  function initCanvas() {
    applyBackground(bgIdx);
    var canvas = document.getElementById('spotlight-canvas');
    var bgImg = new Image();
    var bgReady = false;
    window.thrallSetBgIdx = function (idx) {
      var n = ((idx - 1 + 5) % 5) + 1;
      bgIdx = n;
      window.thrallBgIdx = n;
      try {
        sessionStorage.setItem('thrall_bg_idx', String(n));
      } catch (e) {}
      applyBackground(n);
      bgReady = false;
      bgImg.src = '/bg-' + n + '.jpg';
      if (window.thrallRenderSpotlight) window.thrallRenderSpotlight();
    };
    if (!canvas) return;
    var ctx = canvas.getContext('2d', { alpha: false });
    if (!ctx) return;

    bgImg.onload = function () {
      bgReady = true;
      wakeRender();
      if (reducedMotion) render(performance.now());
    };
    bgImg.src = '/bg-' + bgIdx + '.jpg';

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

    var TEXT_SEL = 'h1,h2,h3,p,pre,code,label,input,button,a,summary,details,.spotlight-card,.tui-screen-mock,.tenets-marquee-wrap,.footer-dock';

    function isOverTextElement(target) {
      if (!target || target === document.body || target === document.documentElement || target === canvas) return false;
      var node = target.nodeType === 3 ? target.parentElement : target;
      if (!node || typeof node.closest !== 'function') return false;
      if (node.id === 'spotlight-canvas' || node.classList.contains('login-stage') || node.classList.contains('wordmark-hero')) return false;
      return Boolean(node.closest(TEXT_SEL));
    }
    window.ThrallIsOverText = isOverTextElement;

    var reducedMotion = window.matchMedia && window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    if (uiAttr('reduce-motion') === '1') reducedMotion = true;
    var dpr = 1, width = 0, height = 0, cellPx = 8, originX = 0, originY = 0, gridCols = 0, gridRows = 0;
    var wmWidth = THRALL_WORDMARK[0].length, wmHeight = THRALL_WORDMARK.length, wmRows = THRALL_WORDMARK;
    // The stage draws online hubs only. `slots` maps each drawn slot to its roster
    // index, so the roster-ordered telemetry series stay index-aligned.
    var slots = [];
    var rawFlags = canvas.getAttribute('data-hubs-flags');
    if (rawFlags !== null) {
      var flagList = rawFlags.split(',');
      for (var fl = 0; fl < flagList.length; fl++) if (flagList[fl].trim() === 'true') slots.push(fl);
    } else {
      var fallbackHubs = Math.max(0, parseInt(canvas.getAttribute('data-hubs-online') || '0', 10) || 0);
      for (var fb = 0; fb < fallbackHubs; fb++) slots.push(fb);
    }
    var meshCount = slots.length;
    var rosterIndex = function (slot) { return slots[slot] !== undefined ? slots[slot] : slot; };
    var rawMini = parseInt(canvas.getAttribute('data-devices'), 10);
    var miniCount = isNaN(rawMini) ? 0 : Math.max(0, rawMini);
    var rawRtts = (canvas.getAttribute('data-mesh-rtts') || '').split(',');
    var meshRtts = [];
    for (var ri = 0; ri < meshCount; ri++) {
      var rv = parseFloat(rawRtts[rosterIndex(ri)]);
      meshRtts.push(isNaN(rv) || rv <= 0 ? (ri + 1) * 14 : rv);
    }
    var rawBps = (canvas.getAttribute('data-mesh-bps') || '').split(',');
    var meshBps = [], totalBps = 0;
    for (var bi = 0; bi < meshCount; bi++) {
      var bv = parseFloat(rawBps[rosterIndex(bi)]);
      var cleanB = isNaN(bv) || bv < 0 ? 0 : bv;
      meshBps.push(cleanB);
      totalBps += cleanB;
    }
    var rawMasses = (canvas.getAttribute('data-mesh-masses') || '').split(',');
    var meshMasses = [];
    for (var mi = 0; mi < meshCount; mi++) {
      var mv = parseFloat(rawMasses[rosterIndex(mi)]);
      meshMasses.push(isNaN(mv) || mv <= 0 ? (mi + 1) * 1024 : mv);
    }
    var rawBearings = (canvas.getAttribute('data-mesh-bearings') || '').split(',');
    var meshBearings = [];
    for (var bri = 0; bri < meshCount; bri++) {
      var brv = parseFloat(rawBearings[rosterIndex(bri)]);
      var deg = isNaN(brv) ? (meshCount > 0 ? (bri * 360 / meshCount) : 0) : brv;
      meshBearings.push((deg * Math.PI) / 180);
    }
    var rawParents = (canvas.getAttribute('data-device-parents') || '').split(',');
    var miniParents = [];
    for (var pi = 0; pi < miniCount; pi++) {
      var pv = parseInt(rawParents[pi], 10);
      var slot = slots.indexOf(pv);
      if (slot < 0) slot = pi;
      miniParents.push(meshCount > 0 ? (slot % meshCount) : -1);
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
      if (bgReady && bgImg.naturalWidth > 0 && bgImg.naturalHeight > 0) {
        var imgScale = Math.max(width / bgImg.naturalWidth, height / bgImg.naturalHeight);
        var dw = bgImg.naturalWidth * imgScale;
        var dh = bgImg.naturalHeight * imgScale;
        var dxBg = (width - dw) * 0.5;
        ctx.save();
        ctx.globalAlpha = 0.14;
        ctx.drawImage(bgImg, dxBg, 0, dw, dh);
        ctx.restore();
      }

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
        ctx.restore();
      }
      for (var s = 0; s < miniCount; s++) {
        var sx = Math.round(miniOrbs[s].x / cellPx) * cellPx;
        var sy = Math.round(miniOrbs[s].y / cellPx) * cellPx;
        ctx.save();
        ctx.fillStyle = '#00e5ff';
        ctx.fillRect(sx, sy, cellPx - 1, cellPx - 1);
        ctx.fillStyle = '#ffffff';
        var inner = Math.max(2, Math.floor(cellPx * 0.45));
        var off = Math.floor((cellPx - 1 - inner) * 0.5);
        ctx.fillRect(sx + off, sy + off, inner, inner);
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
      // d.online is roster-ordered; rebuild the drawn-slot mapping from it.
      if (Array.isArray(d.online)) {
        slots = [];
        for (var oi = 0; oi < d.online.length; oi++) if (d.online[oi]) slots.push(oi);
        meshCount = slots.length;
      } else if (typeof d.hubs_online === 'number') {
        meshCount = Math.max(0, d.hubs_online);
        slots = [];
        for (var oi2 = 0; oi2 < meshCount; oi2++) slots.push(oi2);
      }
      if (typeof d.spokes === 'number' && d.spokes >= 0) miniCount = d.spokes;
      if (Array.isArray(d.rtts)) {
        for (var i = 0; i < meshCount; i++) {
          var rv = parseFloat(d.rtts[rosterIndex(i)]);
          meshRtts[i] = isNaN(rv) || rv <= 0 ? (i + 1) * 14 : rv;
        }
      }
      if (Array.isArray(d.bps)) {
        totalBps = 0;
        for (var j = 0; j < meshCount; j++) {
          var bv = parseFloat(d.bps[rosterIndex(j)]);
          var cb = isNaN(bv) || bv < 0 ? 0 : bv;
          meshBps[j] = cb;
          totalBps += cb;
        }
      }
      if (Array.isArray(d.masses)) {
        for (var k = 0; k < meshCount; k++) {
          var mv = parseFloat(d.masses[rosterIndex(k)]);
          meshMasses[k] = isNaN(mv) || mv <= 0 ? (k + 1) * 1024 : mv;
        }
      }
      if (Array.isArray(d.bearings)) {
        for (var b = 0; b < meshCount; b++) {
          var brv = parseFloat(d.bearings[rosterIndex(b)]);
          var deg = isNaN(brv) ? (meshCount > 0 ? (b * 360 / meshCount) : 0) : brv;
          meshBearings[b] = (deg * Math.PI) / 180;
        }
      }
      if (Array.isArray(d.parents)) {
        for (var p = 0; p < miniCount; p++) {
          var pv = parseInt(d.parents[p], 10);
          var slot = slots.indexOf(pv);
          if (slot < 0) slot = p;
          miniParents[p] = meshCount > 0 ? (slot % meshCount) : -1;
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

  function padRight(str, len) {
    var s = String(str);
    while (s.length < len) s += ' ';
    return s;
  }

  var browserBytesInWindow = 0;
  function recordBrowserBytes(bytes) {
    var n = Number(bytes) || 0;
    if (n > 0) browserBytesInWindow += n;
  }
  window.thrallRecordBrowserBytes = recordBrowserBytes;

  var PIXEL_GLYPHS_3X5 = {
    '0': ['111', '101', '101', '101', '111'],
    '1': ['010', '110', '010', '010', '111'],
    '2': ['111', '001', '111', '100', '111'],
    '3': ['111', '001', '111', '001', '111'],
    '4': ['101', '101', '111', '001', '001'],
    '5': ['111', '100', '111', '001', '111'],
    '6': ['111', '100', '111', '101', '111'],
    '7': ['111', '001', '001', '010', '010'],
    '8': ['111', '101', '111', '101', '111'],
    '9': ['111', '101', '111', '001', '111'],
    'A': ['010', '101', '111', '101', '101'],
    'B': ['110', '101', '110', '101', '110'],
    'C': ['111', '100', '100', '100', '111'],
    'D': ['110', '101', '101', '101', '110'],
    'E': ['111', '100', '110', '100', '111'],
    'F': ['111', '100', '110', '100', '100'],
    'G': ['111', '100', '101', '101', '111'],
    'H': ['101', '101', '111', '101', '101'],
    'I': ['111', '010', '010', '010', '111'],
    'J': ['001', '001', '001', '101', '111'],
    'K': ['101', '101', '110', '101', '101'],
    'L': ['100', '100', '100', '100', '111'],
    'M': ['101', '111', '111', '101', '101'],
    'N': ['101', '111', '111', '111', '101'],
    'O': ['111', '101', '101', '101', '111'],
    'P': ['111', '101', '111', '100', '100'],
    'Q': ['111', '101', '101', '111', '001'],
    'R': ['110', '101', '110', '101', '101'],
    'S': ['111', '100', '111', '001', '111'],
    'T': ['111', '010', '010', '010', '010'],
    'U': ['101', '101', '101', '101', '111'],
    'V': ['101', '101', '101', '101', '010'],
    'W': ['101', '101', '111', '111', '101'],
    'X': ['101', '101', '010', '101', '101'],
    'Y': ['101', '101', '010', '010', '010'],
    'Z': ['111', '001', '010', '100', '111'],
    '%': ['101', '001', '010', '100', '101'],
    '/': ['001', '001', '010', '100', '100'],
    ':': ['000', '010', '000', '010', '000'],
    '-': ['000', '000', '111', '000', '000'],
    '.': ['000', '000', '000', '000', '010'],
    ' ': ['000', '000', '000', '000', '000']
  };

  function initTelemetryPolling() {
    function poll() {
      if (document.hidden || typeof window.fetch !== 'function') return;
      fetch('/api/telemetry', { cache: 'no-store', credentials: 'same-origin' })
        .then(function (r) {
          return r.ok ? r.text() : null;
        })
        .then(function (txt) {
          if (!txt) return;
          recordBrowserBytes(txt.length);
          var d = null;
          try {
            d = JSON.parse(txt);
          } catch (e) {
            return;
          }
          if (!d) return;
          if (window.thrallApplyLiveTelemetry) window.thrallApplyLiveTelemetry(d);
          if (window.thrallUpdateFooterTelemetry) window.thrallUpdateFooterTelemetry(d);
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
        })
        .catch(function () {});
    }
    setTimeout(poll, 900);
    setInterval(poll, 15000);
  }

  function initExplorerTree() {
    var STORAGE_KEY = 'thrall_expanded_dirs';
    function loadExpanded() {
      try {
        var raw = sessionStorage.getItem(STORAGE_KEY);
        var arr = raw ? JSON.parse(raw) : [];
        return Array.isArray(arr) ? arr : [];
      } catch (e) {
        return [];
      }
    }
    function saveExpanded(list) {
      try {
        sessionStorage.setItem(STORAGE_KEY, JSON.stringify(list));
      } catch (e) {}
    }
    function getDirectChildren(targetKey) {
      var rows = document.querySelectorAll('tr.tree-child[data-parent]');
      var out = [];
      for (var i = 0; i < rows.length; i++) {
        if (rows[i].getAttribute('data-parent') === targetKey) out.push(rows[i]);
      }
      return out;
    }
    function setBranchExpanded(btn, expand, persist) {
      if (!btn) return;
      var targetKey = btn.getAttribute('data-target');
      if (!targetKey) return;
      btn.setAttribute('aria-expanded', expand ? 'true' : 'false');
      btn.textContent = expand ? '\u2212' : '+';
      var kids = getDirectChildren(targetKey);
      for (var i = 0; i < kids.length; i++) {
        kids[i].style.display = expand ? '' : 'none';
      }
      if (!expand) {
        var prefix = targetKey + ':';
        var openSub = document.querySelectorAll('.tree-toggle[aria-expanded="true"]');
        for (var j = 0; j < openSub.length; j++) {
          var subKey = openSub[j].getAttribute('data-target') || '';
          if (subKey.indexOf(prefix) === 0) {
            setBranchExpanded(openSub[j], false, false);
          }
        }
      }
      if (persist) {
        var domToggles = document.querySelectorAll('.tree-toggle[data-target]');
        var domMap = {};
        for (var d = 0; d < domToggles.length; d++) {
          var dk = domToggles[d].getAttribute('data-target');
          if (dk) domMap[dk] = domToggles[d].getAttribute('aria-expanded') === 'true';
        }
        var existing = loadExpanded();
        var merged = [];
        for (var e = 0; e < existing.length; e++) {
          var ek = existing[e];
          if (!(ek in domMap) && merged.indexOf(ek) === -1) {
            merged.push(ek);
          }
        }
        for (var k in domMap) {
          if (Object.prototype.hasOwnProperty.call(domMap, k) && domMap[k] && merged.indexOf(k) === -1) {
            merged.push(k);
          }
        }
        saveExpanded(merged);
      }
    }
    function restoreExpandedFromStorage() {
      var saved = loadExpanded();
      if (saved.length) {
        for (var s = 0; s < saved.length; s++) {
          var allBtns = document.querySelectorAll('.tree-toggle[data-target]');
          for (var b = 0; b < allBtns.length; b++) {
            if (allBtns[b].getAttribute('data-target') === saved[s]) {
              setBranchExpanded(allBtns[b], true, false);
            }
          }
        }
      }
    }
    window.thrallRestoreTreeState = restoreExpandedFromStorage;
    restoreExpandedFromStorage();
    document.addEventListener('click', function (e) {
      var t = e.target;
      if (!t || typeof t.closest !== 'function') return;
      var btn = t.closest('.tree-toggle[data-target]');
      if (!btn) return;
      e.preventDefault();
      var isExp = btn.getAttribute('aria-expanded') === 'true';
      setBranchExpanded(btn, !isExp, true);
    });
    document.addEventListener('change', function (e) {
      var t = e.target;
      if (!t || !t.matches || !t.matches('form[action="/admin/shared/update"] input[type="checkbox"]')) return;
      var form = t.closest('form[action="/admin/shared/update"]');
      if (!form) return;
      var boxes = form.querySelectorAll('input[type="checkbox"]');
      var isDirty = false;
      for (var i = 0; i < boxes.length; i++) {
        if (boxes[i].checked !== boxes[i].defaultChecked) {
          isDirty = true;
          break;
        }
      }
      var saveBtn = form.querySelector('.acl-save-btn');
      if (saveBtn) {
        var defLbl = saveBtn.getAttribute('data-default-label') || 'Save Access';
        saveBtn.classList.toggle('dirty-pulse', isDirty);
        saveBtn.classList.toggle('btn-secondary', !isDirty);
        saveBtn.textContent = isDirty ? '\u26A0 Save Changes *' : defLbl;
      }
      var anyFormDirty = Boolean(document.querySelector('.acl-save-btn.dirty-pulse'));
      if (window.thrallShowToast) {
        if (anyFormDirty) {
          window.thrallShowToast('Unsaved access changes \u2014 click Save Changes * to apply', false, false);
        } else {
          window.thrallShowToast('', false, false);
        }
      }
    });
  }

  function escHtml(s) {
    return String(s || '')
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#39;');
  }

  function initExplorerModals() {
    var backdrop = null;

    function closeModal() {
      if (backdrop && backdrop.parentNode) {
        backdrop.parentNode.removeChild(backdrop);
      }
      backdrop = null;
    }

    function openModal(html, isDanger, onReady) {
      closeModal();
      backdrop = document.createElement('div');
      backdrop.id = 'thrall-modal-backdrop';
      backdrop.className = 'thrall-modal-backdrop';
      var card = document.createElement('div');
      card.className = 'thrall-modal' + (isDanger ? ' modal-danger' : '');
      card.setAttribute('role', 'dialog');
      card.setAttribute('aria-modal', 'true');
      card.innerHTML = html;
      backdrop.appendChild(card);
      document.body.appendChild(backdrop);

      backdrop.addEventListener('click', function (e) {
        if (e.target === backdrop || (e.target && e.target.closest && e.target.closest('[data-modal-close]'))) {
          closeModal();
        }
      });

      if (typeof onReady === 'function') onReady(card);
    }

    document.addEventListener('keydown', function (e) {
      if (e.key === 'Escape' && backdrop) {
        e.preventDefault();
        closeModal();
      }
    });

    document.addEventListener('click', function (e) {
      var t = e.target;
      if (!t || typeof t.closest !== 'function') return;

      var delBtn = t.closest('[data-del-action]');
      if (delBtn) {
        e.preventDefault();
        var action = delBtn.getAttribute('data-del-action') || '/files/delete';
        var name = delBtn.getAttribute('data-del-name') || '';
        var browse = delBtn.getAttribute('data-del-browse') || '';
        var sub = delBtn.getAttribute('data-del-sub') || '';
        var item = delBtn.getAttribute('data-del-item') || '';
        var html =
          '<h3>Confirm Deletion</h3>' +
          '<p style="font-size:0.84rem;color:var(--text-secondary);margin-bottom:0.75rem">Type <code style="color:var(--danger);font-weight:700">' + escHtml(name) + '</code> to confirm:</p>' +
          '<form method="POST" action="' + escHtml(action) + '">' +
          '<input type="hidden" name="folder" value="' + escHtml(name) + '">' +
          '<input type="hidden" name="browse" value="' + escHtml(browse) + '">' +
          '<input type="hidden" name="sub" value="' + escHtml(sub) + '">' +
          '<input type="hidden" name="item" value="' + escHtml(item) + '">' +
          '<input type="hidden" name="device_id" value="' + escHtml(item) + '">' +
          '<div style="margin-bottom:0.85rem"><input type="text" name="confirm_name" id="thrall-del-confirm-input" autocomplete="off" required placeholder="Type exact name to confirm"></div>' +
          '<div class="row" style="justify-content:flex-end;gap:0.5rem">' +
          '<button type="button" class="btn-secondary" data-modal-close="1">Cancel</button>' +
          '<button type="submit" class="btn-danger" id="thrall-del-submit-btn" disabled>Confirm</button>' +
          '</div></form>';
        openModal(html, true, function (card) {
          var form = card.querySelector('form');
          var inp = card.querySelector('#thrall-del-confirm-input');
          var subBtn = card.querySelector('#thrall-del-submit-btn');
          if (inp && subBtn) {
            inp.focus();
            inp.addEventListener('input', function () {
              subBtn.disabled = inp.value.trim() !== name;
            });
          }
          if (form && inp) {
            form.addEventListener('submit', function (ev) {
              if (inp.value.trim() !== name) {
                ev.preventDefault();
              }
            });
          }
        });
        return;
      }

      var openBtn = t.closest('[data-modal-open]');
      if (!openBtn) return;
      e.preventDefault();
      var kind = openBtn.getAttribute('data-modal-open');
      var br = openBtn.getAttribute('data-browse') || '';
      var sb = openBtn.getAttribute('data-sub') || '';
      var rbr = openBtn.getAttribute('data-ret-br') || '';
      var lbl = openBtn.getAttribute('data-label') || '';

      if (kind === 'about-thrall') {
        openModal(
          '<h3>About THRALL</h3>' +
          '<p style="font-size:0.82rem;color:var(--text-secondary);line-height:1.55;margin-bottom:0.75rem">' +
          '<strong>THRALL</strong> (<em>Tool for Hierarchical Replication And Identity Linking</em>) is a zero-trust multi-server Syncthing control plane and self-service web portal written in pure-std Rust.' +
          '</p>' +
          '<div class="grid" style="gap:0.5rem;margin-bottom:0.9rem;font-size:0.78rem;font-family:var(--mono)">' +
          '<a href="https://github.com/UberMetroid/thrall" target="_blank" rel="noopener" class="user-chip" style="justify-content:space-between;text-decoration:none;color:var(--accent)"><span>&#128214; THRALL GitHub Repository</span><span>&nearr;</span></a>' +
          '<a href="https://syncthing.net/" target="_blank" rel="noopener" class="user-chip" style="justify-content:space-between;text-decoration:none;color:var(--text)"><span>&#128279; Syncthing &amp; BEP Protocol</span><span>&nearr;</span></a>' +
          '<a href="https://github.com/UberMetroid/tuition-not-coffee" target="_blank" rel="noopener" class="user-chip" style="justify-content:space-between;text-decoration:none;color:var(--warn)"><span>&#127891; Tuition, Not Coffee (College Fund)</span><span>&nearr;</span></a>' +
          '<a href="https://github.com/UberMetroid/thrall/blob/main/AGENTS.md" target="_blank" rel="noopener" class="user-chip" style="justify-content:space-between;text-decoration:none;color:var(--text-secondary)"><span>&#128737; Architecture &amp; Rules (Zero-Dependency Rust)</span><span>&nearr;</span></a>' +
          '</div>' +
          '<div class="row" style="justify-content:flex-end">' +
          '<button type="button" class="btn-secondary" data-modal-close="1">Close</button>' +
          '</div>',
          false
        );
      } else if (kind === 'create-standard-folder') {
        var existingRaw = (openBtn.getAttribute('data-existing') || '').split(',').map(function (s) { return s.trim().toLowerCase(); }).filter(Boolean);
        var xdgAll = ['Desktop', 'Documents', 'Downloads', 'Music', 'Pictures', 'Videos', 'Public', 'Templates'];
        var xdgCommon = ['Desktop', 'Documents', 'Downloads', 'Music', 'Pictures', 'Videos'];
        var xdgHtml = '';
        for (var xi = 0; xi < xdgAll.length; xi++) {
          var xf = xdgAll[xi];
          var hasIt = existingRaw.indexOf(xf.toLowerCase()) !== -1;
          var preChk = hasIt || xdgCommon.indexOf(xf) !== -1;
          if (hasIt) {
            xdgHtml += '<label class="user-chip" style="opacity:0.65;cursor:default"><input type="checkbox" checked disabled>' + escHtml(xf) + ' <span style="color:var(--phosphor);font-size:0.7rem">&#10003; active</span></label>';
          } else {
            xdgHtml += '<label class="user-chip"><input type="checkbox" name="xdg" value="' + escHtml(xf) + '"' + (preChk ? ' checked' : '') + '>' + escHtml(xf) + '</label>';
          }
        }
        openModal(
          '<h3>Add Standard Folders (XDG)</h3>' +
          '<form method="POST" action="/user/folders/create">' +
          '<div class="acl-user-grid" style="grid-template-columns:repeat(2,minmax(0,1fr));margin-bottom:0.75rem">' + xdgHtml + '</div>' +
          '<div style="font-size:0.74rem;color:var(--text-secondary);margin-bottom:0.85rem;display:flex;gap:0.75rem;flex-wrap:wrap;align-items:center">' +
          '<span>Sync selected XDG to:</span>' +
          '<label style="display:inline-flex;align-items:center;gap:0.3rem;cursor:pointer;font-family:var(--mono)"><input type="radio" name="xdg_root" value="home" checked> <code>~/&lt;Folder&gt;</code></label>' +
          '<label style="display:inline-flex;align-items:center;gap:0.3rem;cursor:pointer;font-family:var(--mono)"><input type="radio" name="xdg_root" value="thrall"> <code>~/Thrall/&lt;Folder&gt;</code></label>' +
          '</div>' +
          '<div class="row" style="justify-content:flex-end;gap:0.5rem">' +
          '<button type="button" class="btn-secondary" data-modal-close="1">Cancel</button>' +
          '<button type="submit">Add Selected Folders</button>' +
          '</div></form>',
          false
        );
      } else if (kind === 'create-custom-folder' || kind === 'create-personal-folder') {
        openModal(
          '<h3>Create Custom Folder</h3>' +
          '<form method="POST" action="/user/folders/create">' +
          '<div style="margin-bottom:0.65rem"><input type="text" name="folder" required placeholder="Folder Name (e.g. Projects)"></div>' +
          '<div style="font-size:0.74rem;color:var(--text-secondary);margin-bottom:0.65rem;display:flex;gap:0.75rem;flex-wrap:wrap;align-items:center">' +
          '<span>Sync to:</span>' +
          '<label style="display:inline-flex;align-items:center;gap:0.3rem;cursor:pointer;font-family:var(--mono)"><input type="radio" name="xdg_root" value="thrall" checked> <code>~/Thrall/&lt;Folder&gt;</code></label>' +
          '<label style="display:inline-flex;align-items:center;gap:0.3rem;cursor:pointer;font-family:var(--mono)"><input type="radio" name="xdg_root" value="home"> <code>~/&lt;Folder&gt;</code></label>' +
          '</div>' +
          '<div style="margin-bottom:0.85rem"><input type="text" name="local_path" placeholder="Optional custom local path (e.g. ~/Work/Projects)"></div>' +
          '<div class="row" style="justify-content:flex-end;gap:0.5rem">' +
          '<button type="button" class="btn-secondary" data-modal-close="1">Cancel</button>' +
          '<button type="submit">Create Folder</button>' +
          '</div></form>',
          false,
          function (card) {
            var inp = card.querySelector('input[name="folder"]');
            if (inp) inp.focus();
          }
        );
      } else if (kind === 'set-folder-path') {
        var fScope = openBtn.getAttribute('data-folder') || '';
        var curPath = openBtn.getAttribute('data-path') || '';
        var fShort = fScope.indexOf(':') !== -1 ? fScope.split(':')[1] : fScope;
        var optDirect = '~/' + fShort;
        var optThrall = '~/Thrall/' + fShort;
        openModal(
          '<h3>Local Sync Path <code style="font-size:0.8rem;color:var(--phosphor)">(' + escHtml(fShort) + ')</code></h3>' +
          '<p style="font-size:0.78rem;color:var(--text-secondary);margin-bottom:0.65rem">Choose where this folder syncs on your workstation (Linux, macOS, or Windows):</p>' +
          '<form method="POST" action="/user/folders/path">' +
          '<input type="hidden" name="folder" value="' + escHtml(fScope) + '">' +
          '<div class="row" style="gap:0.45rem;margin-bottom:0.6rem">' +
          '<button type="button" class="btn-secondary" style="padding:0.22rem 0.55rem;font-size:0.74rem;text-transform:none" data-preset-path="' + escHtml(optDirect) + '">Use ' + escHtml(optDirect) + '</button>' +
          '<button type="button" class="btn-secondary" style="padding:0.22rem 0.55rem;font-size:0.74rem;text-transform:none" data-preset-path="' + escHtml(optThrall) + '">Use ' + escHtml(optThrall) + '</button>' +
          '</div>' +
          '<div style="margin-bottom:0.85rem"><input type="text" name="local_path" id="thrall-local-path-input" required value="' + escHtml(curPath) + '" placeholder="e.g. ~/Desktop or ~/Thrall/' + escHtml(fShort) + '"></div>' +
          '<div class="row" style="justify-content:flex-end;gap:0.5rem">' +
          '<button type="button" class="btn-secondary" data-modal-close="1">Cancel</button>' +
          '<button type="submit">Save Path</button>' +
          '</div></form>',
          false,
          function (card) {
            var inp = card.querySelector('#thrall-local-path-input');
            var presets = card.querySelectorAll('[data-preset-path]');
            for (var pi = 0; pi < presets.length; pi++) {
              presets[pi].addEventListener('click', function (ev) {
                var p = ev.currentTarget.getAttribute('data-preset-path');
                if (inp && p) inp.value = p;
              });
            }
            if (inp) inp.focus();
          }
        );
      } else if (kind === 'create-shared-folder') {
        openModal(
          '<h3>Create Shared Folder</h3>' +
          '<form method="POST" action="/admin/shared/create">' +
          '<div style="margin-bottom:0.85rem"><input type="text" name="folder" required placeholder="Shared Folder Name (e.g. Engineering)"></div>' +
          '<div class="row" style="justify-content:flex-end;gap:0.5rem">' +
          '<button type="button" class="btn-secondary" data-modal-close="1">Cancel</button>' +
          '<button type="submit">Create Shared Folder</button>' +
          '</div></form>',
          false,
          function (card) {
            var inp = card.querySelector('input[name="folder"]');
            if (inp) inp.focus();
          }
        );
      } else if (kind === 'add-mesh-hub') {
        openModal(
          '<h3>Add Mesh Hub</h3>' +
          '<form method="POST" action="/admin/servers/add">' +
          '<div style="margin-bottom:0.65rem"><input type="text" name="name" required placeholder="Hub Name (e.g. hyperion)"></div>' +
          '<div style="margin-bottom:0.65rem"><input type="text" name="api_url" required placeholder="Peer API URL (e.g. http://100.x.y.z:8384)"></div>' +
          '<div style="margin-bottom:0.85rem"><input type="text" name="api_key" required placeholder="Peer API Key (from the peer config)"></div>' +
          '<div class="row" style="justify-content:flex-end;gap:0.5rem">' +
          '<button type="button" class="btn-secondary" data-modal-close="1">Cancel</button>' +
          '<button type="submit">Add Hub</button>' +
          '</div></form>',
          false,
          function (card) {
            var inp = card.querySelector('input[name="name"]');
            if (inp) inp.focus();
          }
        );
      } else if (kind === 'enroll-device') {
        var tmpl = document.getElementById('tmpl-enroll-device');
        var gwHtml = tmpl ? tmpl.innerHTML : '';
        var fListRaw = (openBtn.getAttribute('data-folders') || '').split(',').map(function (s) { return s.trim(); }).filter(Boolean);
        var selChecks = '';
        for (var fi = 0; fi < fListRaw.length; fi++) {
          var fk = fListRaw[fi];
          var isShared = fk.indexOf('shared:') === 0;
          var fn = fk.indexOf(':') !== -1 ? fk.split(':')[1] : fk;
          var icon = isShared ? '&#127760; Shared / ' : '&#128193; Personal / ';
          selChecks += '<label class="user-chip"><input type="checkbox" name="sync_folders" value="' + escHtml(fk) + '" checked>' + icon + escHtml(fn) + '</label>';
        }
        var selBlock = fListRaw.length
          ? '<div style="font-size:0.76rem;color:var(--text-secondary);margin-bottom:0.55rem;display:flex;gap:0.85rem;flex-wrap:wrap;align-items:center">' +
            '<span>Initial Sync:</span>' +
            '<label style="display:inline-flex;align-items:center;gap:0.3rem;cursor:pointer;font-family:var(--mono)"><input type="radio" name="sync_mode" value="all" checked> Sync Everything</label>' +
            '<label style="display:inline-flex;align-items:center;gap:0.3rem;cursor:pointer;font-family:var(--mono)"><input type="radio" name="sync_mode" value="selective"> Selective Sync</label>' +
            '</div>' +
            '<div id="thrall-selective-sync-grid" class="acl-user-grid" style="display:none;grid-template-columns:repeat(2,minmax(0,1fr));margin-bottom:0.85rem">' + selChecks + '</div>'
          : '<input type="hidden" name="sync_mode" value="all">';
        openModal(
          '<h3>Add Device (Two-Way Handshake)</h3>' +
          '<div style="font-size:0.8rem;color:var(--text-secondary);margin-bottom:0.75rem">' +
          '<strong>Step 1 (On Your Device):</strong> Open <code>http://127.0.0.1:8384</code> &rarr; <strong>+ Add Remote Device</strong>, paste this Gateway Hub ID, and check <code>[x] Introducer</code> &amp; <code>[x] Auto Accept</code>:' +
          gwHtml +
          '</div>' +
          '<form method="POST" action="/devices/add">' +
          '<div style="font-size:0.8rem;color:var(--text-secondary);margin-bottom:0.45rem"><strong>Step 2 (In This Portal):</strong> Paste your Device\'s 56-character ID (from <em>Actions &rarr; Show ID</em>):</div>' +
          '<div style="margin-bottom:0.65rem"><input type="text" name="device_id" required placeholder="Paste 56-Char Syncthing Device ID"></div>' +
          selBlock +
          '<div class="row" style="justify-content:flex-end;gap:0.5rem">' +
          '<button type="button" class="btn-secondary" data-modal-close="1">Cancel</button>' +
          '<button type="submit">Add Device</button>' +
          '</div></form>',
          false,
          function (card) {
            var inp = card.querySelector('input[name="device_id"]');
            var radios = card.querySelectorAll('input[name="sync_mode"]');
            var grid = card.querySelector('#thrall-selective-sync-grid');
            for (var ri = 0; ri < radios.length; ri++) {
              radios[ri].addEventListener('change', function (ev) {
                if (grid) grid.style.display = ev.target.value === 'selective' ? 'grid' : 'none';
              });
            }
            if (inp) inp.focus();
          }
        );
      } else if (kind === 'mkdir') {
        var subTitle = lbl ? ' <code style="font-size:0.78rem;color:var(--muted)">(' + escHtml(lbl) + ')</code>' : '';
        openModal(
          '<h3>New Subfolder' + subTitle + '</h3>' +
          '<form method="POST" action="/folders/mkdir">' +
          '<input type="hidden" name="browse" value="' + escHtml(br) + '">' +
          '<input type="hidden" name="sub" value="' + escHtml(sb) + '">' +
          '<input type="hidden" name="ret_br" value="' + escHtml(rbr) + '">' +
          '<div style="margin-bottom:0.85rem"><input type="text" name="dirname" required placeholder="Subfolder Name"></div>' +
          '<div class="row" style="justify-content:flex-end;gap:0.5rem">' +
          '<button type="button" class="btn-secondary" data-modal-close="1">Cancel</button>' +
          '<button type="submit">Create Subfolder</button>' +
          '</div></form>',
          false,
          function (card) {
            var inp = card.querySelector('input[name="dirname"]');
            if (inp) inp.focus();
          }
        );
      } else if (kind === 'upload-file') {
        var upTitle = lbl ? ' <code style="font-size:0.78rem;color:var(--muted)">(' + escHtml(lbl) + ')</code>' : '';
        openModal(
          '<h3>Upload Files or Folder' + upTitle + '</h3>' +
          '<div class="upload-dropzone" id="modal-upload-dropzone" data-drop-browse="' + escHtml(br) + '" data-drop-sub="' + escHtml(sb) + '">' +
          'Drag &amp; drop files or entire folders here<br><span style="font-size:0.7rem;color:var(--muted)">or use the buttons below</span>' +
          '</div>' +
          '<input type="file" id="thrall-file-picker" multiple style="display:none">' +
          '<input type="file" id="thrall-dir-picker" webkitdirectory directory multiple style="display:none">' +
          '<div class="row" style="justify-content:space-between;gap:0.5rem">' +
          '<div class="row" style="gap:0.5rem">' +
          '<button type="button" id="thrall-pick-files-btn">+ Choose Files</button>' +
          '<button type="button" class="btn-secondary" id="thrall-pick-dir-btn">+ Choose Folder</button>' +
          '</div>' +
          '<button type="button" class="btn-secondary" data-modal-close="1">Close</button>' +
          '</div>',
          false,
          function (card) {
            var fPick = card.querySelector('#thrall-file-picker');
            var dPick = card.querySelector('#thrall-dir-picker');
            var fBtn = card.querySelector('#thrall-pick-files-btn');
            var dBtn = card.querySelector('#thrall-pick-dir-btn');
            if (fBtn && fPick) fBtn.addEventListener('click', function () { fPick.click(); });
            if (dBtn && dPick) dBtn.addEventListener('click', function () { dPick.click(); });
            function handleList(flist) {
              if (!flist || !flist.length) return;
              var items = [];
              for (var i = 0; i < flist.length; i++) {
                var f = flist[i];
                var relPath = f.webkitRelativePath || f.name || '';
                items.push({ file: f, relPath: relPath });
              }
              closeModal();
              if (window.thrallStartBatchUpload) {
                window.thrallStartBatchUpload(br, sb, items);
              }
            }
            if (fPick) fPick.addEventListener('change', function () { handleList(fPick.files); });
            if (dPick) dPick.addEventListener('change', function () { handleList(dPick.files); });
          }
        );
      }
    });
  }

  function initUploadEngine() {
    var graphCanvas = document.getElementById('footer-upload-graph');
    var hubsCanvas = document.getElementById('footer-hubs-eq');
    var meshCanvas = document.getElementById('footer-mesh-graph');
    var vaultCanvas = document.getElementById('footer-vault-gauge');
    var clockCanvas = document.getElementById('footer-uptime-clock');
    var statusMsg = document.getElementById('footer-status-msg');
    var toastTimer = 0;
    var BAR_COUNT = 22;
    var initialBars = [];
    for (var b = 0; b < BAR_COUNT; b++) {
      initialBars.push(0);
    }
    var uploadGraphState = {
      active: false,
      pct: 0,
      bytesPerSec: 0,
      bars: initialBars
    };
    var MESH_BAR_COUNT = 18;
    var meshBars = [];
    for (var mb = 0; mb < MESH_BAR_COUNT; mb++) {
      meshBars.push(0.08);
    }
    var footerTel = {
      hubsOnline: 3,
      hubsTotal: 3,
      spokes: 0,
      activeLinks: 6,
      uptimeH: 1,
      rtts: [14, 28, 42],
      bps: [0, 0, 0],
      totalMeshBps: 0,
      masses: [1024, 2048, 4096],
      tick: 0
    };
    var spotInit = document.getElementById('spotlight-canvas');
    if (spotInit) {
      var initTot = parseInt(spotInit.getAttribute('data-hubs-total') || '3', 10);
      if (initTot > 0) footerTel.hubsTotal = initTot;
      var initOn = parseInt(spotInit.getAttribute('data-hubs-online') || '0', 10);
      footerTel.hubsOnline = isNaN(initOn) ? 0 : Math.max(0, Math.min(initTot, initOn));
      var initSpokes = parseInt(spotInit.getAttribute('data-devices') || '0', 10);
      if (!isNaN(initSpokes)) footerTel.spokes = initSpokes;
      var initRtts = (spotInit.getAttribute('data-mesh-rtts') || '').split(',').map(function (x) { return parseFloat(x); }).filter(function (x) { return !isNaN(x) && x > 0; });
      if (initRtts.length) footerTel.rtts = initRtts;
      var initMass = (spotInit.getAttribute('data-mesh-masses') || '').split(',').map(function (x) { return parseFloat(x); }).filter(function (x) { return !isNaN(x) && x > 0; });
      if (initMass.length) footerTel.masses = initMass;
    }

    var perfObserverAttached = false;
    var perfResourceCursor = 0;
    if (typeof window.PerformanceObserver === 'function') {
      try {
        var obs = new PerformanceObserver(function (list) {
          var entries = list.getEntries ? list.getEntries() : [];
          for (var i = 0; i < entries.length; i++) {
            var en = entries[i];
            var name = (en && en.name) || '';
            var initType = (en && en.initiatorType) || '';
            if (initType === 'fetch' || name.indexOf('/files/upload') !== -1 || name.indexOf('/api/telemetry') !== -1) {
              continue;
            }
            var sz = (en && (en.transferSize || en.encodedBodySize || en.decodedBodySize)) || 0;
            if (sz > 0) recordBrowserBytes(sz);
          }
        });
        obs.observe({ entryTypes: ['resource'] });
        perfObserverAttached = true;
      } catch (e) {
        perfObserverAttached = false;
      }
    }
    if (!perfObserverAttached && typeof window.performance === 'object' && typeof performance.getEntriesByType === 'function') {
      perfResourceCursor = performance.getEntriesByType('resource').length;
    }

    function formatShortRate(bps) {
      var n = Math.max(0, Math.round(bps || 0));
      if (n === 0) return '0B/S';
      if (n < 1024) return n + 'B/S';
      if (n < 1048576) return Math.max(1, Math.round(n / 1024)) + 'K/S';
      return Math.max(1, Math.round(n / 1048576)) + 'M/S';
    }

    function formatExactRate(bps) {
      var n = Math.max(0, Math.round(bps || 0));
      if (n < 1024) return n + ' B/s';
      if (n < 1048576) return (n / 1024).toFixed(1) + ' KB/s';
      return (n / 1048576).toFixed(2) + ' MB/s';
    }

    function drawPixelText3x5(ctx, str, rightX, topY, color) {
      var s = String(str || '').toUpperCase();
      var charW = 3, gap = 1;
      var totalW = s.length > 0 ? (s.length * (charW + gap) - gap) : 0;
      var startX = rightX - totalW;
      ctx.fillStyle = color;
      for (var i = 0; i < s.length; i++) {
        var g = PIXEL_GLYPHS_3X5[s.charAt(i)];
        if (!g) continue;
        var cx = startX + i * (charW + gap);
        for (var r = 0; r < g.length; r++) {
          var rowStr = g[r];
          for (var c = 0; c < rowStr.length; c++) {
            if (rowStr.charAt(c) === '1') {
              ctx.fillRect(cx + c, topY + r, 1, 1);
            }
          }
        }
      }
    }

    function drawPixelTextLeft3x5(ctx, str, leftX, topY, color) {
      var s = String(str || '').toUpperCase();
      var charW = 3, gap = 1;
      ctx.fillStyle = color;
      for (var i = 0; i < s.length; i++) {
        var g = PIXEL_GLYPHS_3X5[s.charAt(i)];
        if (!g) continue;
        var cx = leftX + i * (charW + gap);
        for (var r = 0; r < g.length; r++) {
          var rowStr = g[r];
          for (var c = 0; c < rowStr.length; c++) {
            if (rowStr.charAt(c) === '1') {
              ctx.fillRect(cx + c, topY + r, 1, 1);
            }
          }
        }
      }
    }

    function renderFooterWidgets() {
      var acc = window.thrallCurrentAccent || [80, 250, 123];
      var ar = acc[0], ag = acc[1], ab = acc[2];
      var hr = Math.min(255, Math.round(ar + (255 - ar) * 0.45));
      var hg = Math.min(255, Math.round(ag + (255 - ag) * 0.45));
      var hb = Math.min(255, Math.round(ab + (255 - ab) * 0.45));
      var mainCol = 'rgb(' + ar + ',' + ag + ',' + ab + ')';
      var hotCol = 'rgb(' + hr + ',' + hg + ',' + hb + ')';
      var gridCol = 'rgba(' + ar + ',' + ag + ',' + ab + ',0.14)';

      // #2: 8-Bit Hub Signal / Ping Equalizer
      if (hubsCanvas && hubsCanvas.getContext) {
        var hCtx = hubsCanvas.getContext('2d');
        var hw = hubsCanvas.width, hh = hubsCanvas.height;
        hCtx.fillStyle = '#06080a';
        hCtx.fillRect(0, 0, hw, hh);
        hCtx.fillStyle = gridCol;
        for (var hy = 3; hy < hh - 3; hy += 4) hCtx.fillRect(0, hy, hw, 1);
        var hTot = Math.max(1, Math.min(4, footerTel.hubsTotal || 3));
        var hOn = Math.max(0, Math.min(hTot, footerTel.hubsOnline));
        for (var hi = 0; hi < hTot; hi++) {
          var isOn = hi < hOn;
          var rtt = (footerTel.rtts && footerTel.rtts[hi]) || ((hi + 1) * 14);
          var sigBars = !isOn ? 1 : (rtt < 25 ? 5 : (rtt < 60 ? 4 : (rtt < 120 ? 3 : 2)));
          var tx = 4 + hi * 9;
          hCtx.fillStyle = isOn ? hotCol : '#f85149';
          hCtx.fillRect(tx + 2, 2, 2, 2);
          for (var sb = 0; sb < 5; sb++) {
            var segY = hh - 4 - sb * 3;
            hCtx.fillStyle = sb < sigBars ? (sb === sigBars - 1 ? hotCol : mainCol) : '#14191f';
            hCtx.fillRect(tx, segY, 6, 2);
          }
        }
        var minRtt = (footerTel.rtts && footerTel.rtts.length) ? Math.round(Math.min.apply(null, footerTel.rtts)) : 14;
        drawPixelText3x5(hCtx, hOn + '/' + hTot + 'H', hw - 3, 4, '#ffffff');
        drawPixelText3x5(hCtx, minRtt + 'MS', hw - 3, 13, hotCol);
        hubsCanvas.title = 'Regional Mesh Hubs: ' + hOn + '/' + hTot + ' online (best RTT: ' + minRtt + ' ms)';
      }

      // #3: 8-Bit Mesh Sync Heartbeat / Throughput Graph
      if (meshCanvas && meshCanvas.getContext) {
        var mCtx = meshCanvas.getContext('2d');
        var mw = meshCanvas.width, mh = meshCanvas.height;
        mCtx.fillStyle = '#06080a';
        mCtx.fillRect(0, 0, mw, mh);
        mCtx.fillStyle = gridCol;
        for (var my = 3; my < mh - 4; my += 4) mCtx.fillRect(0, my, mw, 1);
        for (var mi = 0; mi < meshBars.length; mi++) {
          var ml = meshBars[mi] || 0;
          var mbh = ml > 0 ? Math.max(2, Math.min(17, Math.round(ml * 17))) : 0;
          if (mbh <= 0) continue;
          var mbx = 3 + mi * 4;
          for (var mpy = mh - 5; mpy > mh - 5 - mbh; mpy -= 2) {
            mCtx.fillStyle = (mpy <= mh - 5 - mbh + 2) ? hotCol : mainCol;
            mCtx.fillRect(mbx, mpy, 3, 1.5);
          }
        }
        mCtx.fillStyle = '#14191f';
        mCtx.fillRect(2, mh - 3, mw - 4, 2);
        var mBps = footerTel.totalMeshBps || 0;
        var mFill = mBps > 0 ? Math.max(6, Math.round((mw - 4) * Math.min(1, Math.log10(mBps + 1) / 7))) : ((footerTel.tick * 4) % (mw - 8));
        mCtx.fillStyle = mBps > 0 ? hotCol : mainCol;
        mCtx.fillRect(2, mh - 3, mFill, 2);
        drawPixelTextLeft3x5(mCtx, 'MSH', 3, 4, mainCol);
        drawPixelText3x5(mCtx, mBps > 0 ? formatShortRate(mBps) : 'SYNC', mw - 3, 4, '#ffffff');
      }

      // #4: 8-Bit Vault / Mass Gauge
      if (vaultCanvas && vaultCanvas.getContext) {
        var vCtx = vaultCanvas.getContext('2d');
        var vw = vaultCanvas.width, vh = vaultCanvas.height;
        vCtx.fillStyle = '#06080a';
        vCtx.fillRect(0, 0, vw, vh);
        var totalMass = 0;
        for (var vmi = 0; vmi < (footerTel.masses || []).length; vmi++) {
          totalMass += (footerTel.masses[vmi] || 0);
        }
        var massStr = totalMass >= 1073741824
          ? Math.round(totalMass / 1073741824) + 'G'
          : (totalMass >= 1048576 ? Math.round(totalMass / 1048576) + 'M' : Math.max(1, Math.round(totalMass / 1024)) + 'K');
        drawPixelTextLeft3x5(vCtx, 'VLT', 3, 4, hotCol);
        drawPixelText3x5(vCtx, (footerTel.activeLinks || 0) + 'L/' + massStr, vw - 3, 4, '#ffffff');
        var segs = 10;
        var activeSegs = Math.max(2, Math.min(segs, Math.round(Math.log10(totalMass + 10) * 1.1)));
        for (var vs = 0; vs < segs; vs++) {
          vCtx.fillStyle = vs < activeSegs ? (vs === activeSegs - 1 ? hotCol : mainCol) : '#14191f';
          vCtx.fillRect(3 + vs * 8.5, 13, 7, 7);
        }
        vaultCanvas.title = 'Mesh Vault Mass: ' + massStr + ' (' + (footerTel.activeLinks || 0) + ' active BEP links, ' + (footerTel.spokes || 0) + ' devices)';
      }

      // #5: 8-Bit Uptime Clock
      if (clockCanvas && clockCanvas.getContext) {
        var cCtx = clockCanvas.getContext('2d');
        var cw = clockCanvas.width, ch = clockCanvas.height;
        cCtx.fillStyle = '#06080a';
        cCtx.fillRect(0, 0, cw, ch);
        var hgSprite = [
          '11111',
          '01110',
          '00100',
          '01010',
          '11111'
        ];
        for (var hrIdx = 0; hrIdx < hgSprite.length; hrIdx++) {
          for (var hcIdx = 0; hcIdx < 5; hcIdx++) {
            if (hgSprite[hrIdx].charAt(hcIdx) === '1') {
              cCtx.fillStyle = hrIdx === 0 || hrIdx === 4 ? hotCol : mainCol;
              cCtx.fillRect(4 + hcIdx * 2, 4 + hrIdx * 3, 2, 2);
            }
          }
        }
        var upVal = Math.max(1, footerTel.uptimeH || 1);
        drawPixelText3x5(cCtx, 'UP ' + upVal + 'H', cw - 4, 6, '#ffffff');
        var pulseX = 18 + ((Math.floor(footerTel.tick / 2)) % Math.max(4, cw - 22));
        cCtx.fillStyle = '#14191f';
        cCtx.fillRect(18, ch - 4, cw - 22, 2);
        cCtx.fillStyle = hotCol;
        cCtx.fillRect(pulseX, ch - 4, 4, 2);
      }
    }
    window.thrallRenderFooterWidgets = renderFooterWidgets;

    window.thrallUpdateFooterTelemetry = function (d) {
      if (!d || typeof d !== 'object') return;
      if (typeof d.hubs_online === 'number') footerTel.hubsOnline = d.hubs_online;
      if (typeof d.hubs_total === 'number' && d.hubs_total >= 1) footerTel.hubsTotal = d.hubs_total;
      if (typeof d.spokes === 'number') footerTel.spokes = d.spokes;
      if (typeof d.active_links === 'number') footerTel.activeLinks = d.active_links;
      if (typeof d.uptime_h === 'number') footerTel.uptimeH = d.uptime_h;
      if (Array.isArray(d.rtts) && d.rtts.length) footerTel.rtts = d.rtts;
      if (Array.isArray(d.masses) && d.masses.length) footerTel.masses = d.masses;
      if (Array.isArray(d.bps)) {
        footerTel.bps = d.bps;
        var sumB = 0;
        for (var i = 0; i < d.bps.length; i++) sumB += (Number(d.bps[i]) || 0);
        footerTel.totalMeshBps = sumB;
      }
      renderFooterWidgets();
    };

    function showToast(text, isErr, autoHide) {
      if (!statusMsg) statusMsg = document.getElementById('footer-status-msg');
      if (!statusMsg) return;
      if (toastTimer) {
        clearTimeout(toastTimer);
        toastTimer = 0;
      }
      statusMsg.textContent = text;
      statusMsg.style.display = text ? 'inline-flex' : 'none';
      statusMsg.classList.toggle('toast-err', Boolean(isErr));
      if (autoHide && text) {
        toastTimer = setTimeout(function () {
          statusMsg.style.display = 'none';
        }, 4500);
      }
    }
    window.thrallShowToast = showToast;

    if (statusMsg && statusMsg.textContent && statusMsg.style.display !== 'none') {
      toastTimer = setTimeout(function () {
        statusMsg.style.display = 'none';
      }, 4500);
    }

    function cleanSeg(s, isFile) {
      var trimmed = String(s || '').replace(/^\.+/, '');
      var cleaned = isFile
        ? trimmed.replace(/[^a-zA-Z0-9._-]/g, '_').slice(0, 128)
        : trimmed.replace(/[^a-zA-Z0-9_-]/g, '-').slice(0, 64);
      return cleaned || (isFile ? 'file' : 'folder');
    }

    function splitRelPath(baseSub, relPath) {
      var parts = String(relPath || '').split('/').filter(function (p) {
        return p && p !== '.' && p !== '..' && p.charAt(0) !== '.';
      });
      if (!parts.length) return null;
      var rawFile = parts.pop();
      var fileName = cleanSeg(rawFile, true);
      var subParts = [];
      if (baseSub) {
        var bp = String(baseSub).split('/').filter(Boolean);
        for (var i = 0; i < bp.length; i++) subParts.push(bp[i]);
      }
      for (var j = 0; j < parts.length; j++) {
        subParts.push(cleanSeg(parts[j], false));
      }
      return { sub: subParts.join('/'), filename: fileName };
    }

    function bytesToHex(uint8) {
      var out = '';
      for (var i = 0; i < uint8.length; i++) {
        var h = uint8[i].toString(16);
        out += h.length === 1 ? '0' + h : h;
      }
      return out;
    }

    function renderGraphFrame() {
      if (!graphCanvas || !graphCanvas.getContext) return;
      var ctx = graphCanvas.getContext('2d');
      if (!ctx) return;
      var w = graphCanvas.width, h = graphCanvas.height;
      ctx.fillStyle = '#06080a';
      ctx.fillRect(0, 0, w, h);

      var acc = window.thrallCurrentAccent || [80, 250, 123];
      var ar = acc[0], ag = acc[1], ab = acc[2];
      var hr = Math.min(255, Math.round(ar + (255 - ar) * 0.45));
      var hg = Math.min(255, Math.round(ag + (255 - ag) * 0.45));
      var hb = Math.min(255, Math.round(ab + (255 - ab) * 0.45));

      ctx.fillStyle = 'rgba(' + ar + ',' + ag + ',' + ab + ',0.14)';
      for (var gy = 3; gy < h - 4; gy += 4) {
        ctx.fillRect(0, gy, w, 1);
      }

      var barW = 3, gap = 1, startX = 3;
      var bars = uploadGraphState.bars;
      for (var i = 0; i < bars.length; i++) {
        var level = bars[i] || 0;
        var bh = level > 0 ? Math.max(2, Math.min(18, Math.round(level * 18))) : 0;
        if (bh <= 0) continue;
        var bx = startX + i * (barW + gap);
        for (var py = h - 5; py > h - 5 - bh; py -= 2) {
          var isTop = py <= h - 5 - bh + 2;
          ctx.fillStyle = isTop
            ? ('rgb(' + hr + ',' + hg + ',' + hb + ')')
            : ('rgb(' + ar + ',' + ag + ',' + ab + ')');
          ctx.fillRect(bx, py, barW, 1.5);
        }
      }

      ctx.fillStyle = '#14191f';
      ctx.fillRect(2, h - 3, w - 4, 2);

      var bps = uploadGraphState.bytesPerSec || 0;
      if (uploadGraphState.active) {
        var pctClamped = Math.max(0, Math.min(100, Math.round(uploadGraphState.pct)));
        var fillW = Math.round(((w - 4) * pctClamped) / 100);
        ctx.fillStyle = 'rgb(' + hr + ',' + hg + ',' + hb + ')';
        ctx.fillRect(2, h - 3, fillW, 2);
        drawPixelText3x5(ctx, pctClamped + '%', w - 3, 4, '#ffffff');
        graphCanvas.title = 'Browser TX/RX: ' + formatExactRate(bps) + ' (Upload ' + pctClamped + '%)';
      } else {
        if (bps > 0) {
          var rateRatio = Math.min(1, Math.log10(bps + 1) / 7);
          var rateFillW = Math.max(4, Math.round((w - 4) * rateRatio));
          ctx.fillStyle = 'rgb(' + ar + ',' + ag + ',' + ab + ')';
          ctx.fillRect(2, h - 3, rateFillW, 2);
        }
        var labelColor = bps > 0
          ? '#ffffff'
          : ('rgb(' + hr + ',' + hg + ',' + hb + ')');
        drawPixelText3x5(ctx, formatShortRate(bps), w - 3, 4, labelColor);
        graphCanvas.title = 'Browser TX/RX: ' + formatExactRate(bps);
      }
    }

    function pushBrowserLevel(level) {
      uploadGraphState.bars.shift();
      uploadGraphState.bars.push(Math.max(0, Math.min(1, level)));
    }

    function drawPixelGraph(pct) {
      uploadGraphState.active = true;
      uploadGraphState.pct = pct;
      renderGraphFrame();
    }

    function uploadGraphStart() {
      uploadGraphState.active = true;
      uploadGraphState.pct = 0;
      renderGraphFrame();
    }

    function uploadGraphStop() {
      uploadGraphState.active = false;
      uploadGraphState.pct = 0;
      renderGraphFrame();
    }

    window.thrallRenderFooterGraph = renderGraphFrame;
    renderGraphFrame();
    renderFooterWidgets();

    setInterval(function () {
      if (document.hidden) return;
      var perfDelta = 0;
      if (!perfObserverAttached && typeof window.performance === 'object' && typeof performance.getEntriesByType === 'function') {
        var entries = performance.getEntriesByType('resource');
        if (entries.length < perfResourceCursor) {
          perfResourceCursor = 0;
        }
        for (var i = perfResourceCursor; i < entries.length; i++) {
          var en = entries[i];
          var name = (en && en.name) || '';
          var initType = (en && en.initiatorType) || '';
          var isExplicit =
            initType === 'fetch' ||
            name.indexOf('/files/upload') !== -1 ||
            name.indexOf('/api/telemetry') !== -1;
          if (!isExplicit) {
            var sz = (en && (en.transferSize || en.encodedBodySize || en.decodedBodySize)) || 0;
            if (sz > 0) perfDelta += sz;
          }
        }
        perfResourceCursor = entries.length;
        if (entries.length >= 200 && typeof performance.clearResourceTimings === 'function') {
          performance.clearResourceTimings();
          perfResourceCursor = 0;
        }
      }
      var bytesInTick = browserBytesInWindow + perfDelta;
      browserBytesInWindow = 0;
      var bytesPerSec = Math.round(bytesInTick * (1000 / 150));
      uploadGraphState.bytesPerSec = bytesPerSec;
      var barHeight = 0;
      if (bytesInTick > 0) {
        barHeight = Math.max(0.08, Math.min(1.0, Math.log10(bytesPerSec + 1) / 6.5));
      }
      pushBrowserLevel(barHeight);
      footerTel.tick = (footerTel.tick + 1) >>> 0;
      var mRate = footerTel.totalMeshBps || 0;
      var mLevel = mRate > 0
        ? Math.max(0.14, Math.min(1.0, Math.log10(mRate + 1) / 6.5))
        : (footerTel.hubsOnline > 0 ? (0.1 + 0.08 * Math.sin(footerTel.tick * 0.35)) : 0);
      meshBars.shift();
      meshBars.push(mLevel);
      renderGraphFrame();
      renderFooterWidgets();
    }, 150);

    function readBlobSlice(blob) {
      if (blob && typeof blob.arrayBuffer === 'function') {
        return blob.arrayBuffer();
      }
      return new Promise(function (resolve, reject) {
        var fr = new FileReader();
        fr.onload = function () { resolve(fr.result); };
        fr.onerror = function () { reject(fr.error); };
        fr.readAsArrayBuffer(blob);
      });
    }

    function uploadSingleFile(br, baseSub, item, onChunk) {
      var target = splitRelPath(baseSub, item.relPath || item.file.name);
      if (!target) return Promise.resolve();
      var file = item.file;
      var chunkSize = 12288;
      var total = file.size || 0;
      var offset = 0;
      var isFirst = true;

      function nextChunk() {
        if (offset >= total && !isFirst) return Promise.resolve();
        var end = Math.min(total, offset + chunkSize);
        var blobSlice = file.slice ? file.slice(offset, end) : file;
        var currentAppendix = isFirst ? '0' : '1';
        offset = end;
        isFirst = false;
        return readBlobSlice(blobSlice).then(function (buf) {
          var slice = new Uint8Array(buf || new ArrayBuffer(0));
          var hex = bytesToHex(slice);
          var body =
            'browse=' + encodeURIComponent(br) +
            '&sub=' + encodeURIComponent(target.sub) +
            '&filename=' + encodeURIComponent(target.filename) +
            '&append=' + currentAppendix +
            '&hex=' + hex;
          recordBrowserBytes(body.length);
          return fetch('/files/upload', {
            method: 'POST',
            headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
            body: body,
            credentials: 'same-origin'
          }).then(function (r) {
            if (!r.ok) throw new Error('Upload failed');
            return r.text().then(function (respTxt) {
              recordBrowserBytes((respTxt && respTxt.length) || 0);
              if (onChunk) onChunk(slice.length);
              return nextChunk();
            });
          });
        });
      }
      return nextChunk();
    }

    function startBatchUpload(br, baseSub, items) {
      if (uploadGraphState.active) return;
      var valid = (items || []).filter(function (it) {
        var n = (it && it.file && it.file.name) || '';
        return n && n.charAt(0) !== '.';
      });
      if (!valid.length) return;
      if (!br) {
        var firstRow = document.querySelector('tr[data-drop-browse]');
        if (firstRow) br = firstRow.getAttribute('data-drop-browse') || '';
      }
      if (!br) {
        showToast('Create or select a folder before uploading files.', true, true);
        return;
      }

      var totalBytes = 0;
      for (var i = 0; i < valid.length; i++) {
        totalBytes += (valid[i].file.size || 0);
      }
      var uploadedBytes = 0;
      var idx = 0;
      uploadGraphStart();

      function nextFile() {
        if (idx >= valid.length) {
          uploadGraphState.pct = 100;
          renderGraphFrame();
          showToast('Uploaded ' + valid.length + ' file(s)', false, true);
          setTimeout(function () {
            uploadGraphStop();
            if (window.thrallSpaNavigate) {
              window.thrallSpaNavigate(window.location.pathname + window.location.search, false, true);
            } else {
              window.location.reload();
            }
          }, 260);
          return;
        }
        var cur = valid[idx];
        var fNum = idx + 1;
        idx += 1;
        var basePct = totalBytes > 0 ? (uploadedBytes / totalBytes) * 100 : ((fNum - 1) / valid.length) * 100;
        showToast('UPLOADING ' + fNum + '/' + valid.length + ': ' + (cur.relPath || cur.file.name) + ' (' + Math.round(basePct) + '%)', false, false);
        uploadSingleFile(br, baseSub, cur, function (chunkLen) {
          uploadedBytes += chunkLen;
          var pct = totalBytes > 0 ? (uploadedBytes / totalBytes) * 100 : (fNum / valid.length) * 100;
          drawPixelGraph(pct);
          showToast('UPLOADING ' + fNum + '/' + valid.length + ': ' + (cur.relPath || cur.file.name) + ' (' + Math.round(pct) + '%)', false, false);
        })
          .then(nextFile)
          .catch(function () {
            uploadGraphStop();
            showToast('Upload failed on ' + (cur.relPath || cur.file.name), true, true);
          });
      }
      nextFile();
    }
    window.thrallStartBatchUpload = startBatchUpload;

    function traverseEntry(entry, prefix) {
      return new Promise(function (resolve) {
        if (!entry) return resolve([]);
        if (entry.isFile) {
          entry.file(function (f) {
            resolve([{ file: f, relPath: prefix ? (prefix + '/' + f.name) : f.name }]);
          }, function () { resolve([]); });
        } else if (entry.isDirectory) {
          var dirName = entry.name || '';
          if (dirName.charAt(0) === '.') return resolve([]);
          var nextPrefix = prefix ? (prefix + '/' + dirName) : dirName;
          var reader = entry.createReader();
          var allEntries = [];
          function readBatch() {
            reader.readEntries(function (batch) {
              if (!batch || !batch.length) {
                Promise.all(allEntries.map(function (c) { return traverseEntry(c, nextPrefix); }))
                  .then(function (nested) {
                    var flat = [];
                    for (var i = 0; i < nested.length; i++) flat = flat.concat(nested[i]);
                    resolve(flat);
                  });
              } else {
                allEntries = allEntries.concat(Array.prototype.slice.call(batch));
                readBatch();
              }
            }, function () { resolve([]); });
          }
          readBatch();
        } else {
          resolve([]);
        }
      });
    }

    function collectDataTransferFiles(dt) {
      if (!dt) return Promise.resolve([]);
      if (dt.items && dt.items.length) {
        var entries = [];
        for (var i = 0; i < dt.items.length; i++) {
          var it = dt.items[i];
          if (it.webkitGetAsEntry) {
            var en = it.webkitGetAsEntry();
            if (en) entries.push(en);
          }
        }
        if (entries.length) {
          return Promise.all(entries.map(function (e) { return traverseEntry(e, ''); }))
            .then(function (lists) {
              var out = [];
              for (var k = 0; k < lists.length; k++) out = out.concat(lists[k]);
              return out;
            });
        }
      }
      var raw = dt.files || [];
      var fallback = [];
      for (var j = 0; j < raw.length; j++) {
        fallback.push({ file: raw[j], relPath: raw[j].webkitRelativePath || raw[j].name });
      }
      return Promise.resolve(fallback);
    }

    var activeDropEl = null;
    function clearDropHighlight() {
      if (activeDropEl) {
        activeDropEl.classList.remove('drop-active');
        activeDropEl = null;
      }
    }

    document.addEventListener('dragover', function (e) {
      var t = e.target;
      if (!t || typeof t.closest !== 'function') return;
      var dropEl = t.closest('[data-drop-browse]') || t.closest('.explorer-table');
      if (!dropEl) return;
      e.preventDefault();
      if (e.dataTransfer) e.dataTransfer.dropEffect = 'copy';
      if (activeDropEl !== dropEl) {
        clearDropHighlight();
        activeDropEl = dropEl;
        activeDropEl.classList.add('drop-active');
      }
    });

    document.addEventListener('dragleave', function (e) {
      if (activeDropEl && (!e.relatedTarget || !activeDropEl.contains(e.relatedTarget))) {
        clearDropHighlight();
      }
    });

    document.addEventListener('drop', function (e) {
      var t = e.target;
      if (!t || typeof t.closest !== 'function') return;
      var dropEl = t.closest('[data-drop-browse]') || t.closest('.explorer-table');
      if (!dropEl) return;
      e.preventDefault();
      clearDropHighlight();
      var br = dropEl.getAttribute('data-drop-browse') || '';
      var sb = dropEl.getAttribute('data-drop-sub') || '';
      if (!br) {
        var first = dropEl.querySelector('[data-drop-browse]');
        if (first) {
          br = first.getAttribute('data-drop-browse') || '';
          sb = first.getAttribute('data-drop-sub') || '';
        }
      }
      var backdrop = document.getElementById('thrall-modal-backdrop');
      if (backdrop && backdrop.parentNode) backdrop.parentNode.removeChild(backdrop);
      collectDataTransferFiles(e.dataTransfer).then(function (items) {
        if (items && items.length) {
          var tKey = sb ? (br + ':' + sb) : br;
          var expandBtn = document.querySelector('.tree-toggle[data-target="' + tKey + '"]') || document.querySelector('.tree-toggle[data-target="' + br + '"]');
          if (expandBtn && expandBtn.getAttribute('aria-expanded') === 'false') {
            expandBtn.click();
          }
          startBatchUpload(br, sb, items);
        }
      });
    });
  }

  function initSpaNav() {
    if (typeof window.fetch !== 'function' || typeof window.DOMParser !== 'function') return;

    function syncActiveTabFromHref(href) {
      try {
        var u = new URL(href, window.location.origin);
        var tab = u.searchParams.get('tab') || 'personal';
        var browse = u.searchParams.get('browse') || '';
        if (tab === 'global' || tab === 'mesh' || tab === 'ldap') tab = 'admin';
        if (tab === 'personal' && browse.indexOf('shared:') === 0) tab = 'shared';
        else if (tab === 'shared' && browse.indexOf('personal:') === 0) tab = 'personal';
        var tabs = document.querySelectorAll('.body-tabs .body-tab');
        for (var i = 0; i < tabs.length; i++) {
          var th = tabs[i].getAttribute('href') || '';
          tabs[i].classList.toggle('active', th === '/?tab=' + tab);
        }
      } catch (e) {}
    }

    function applyParsedDoc(htmlText, finalUrl, pushHistory, keepToast) {
      var parser = new DOMParser();
      var doc = parser.parseFromString(htmlText, 'text/html');
      var newContainer = doc.querySelector('#middle-scroll .container');
      var curContainer = document.querySelector('#middle-scroll .container');
      if (!newContainer || !curContainer) {
        window.location.href = finalUrl || '/';
        return;
      }
      curContainer.innerHTML = newContainer.innerHTML;

      if (!keepToast) {
        var newToast = doc.getElementById('footer-status-msg');
        if (newToast && window.thrallShowToast) {
          var txt = (newToast.textContent || '').trim();
          var isErr = newToast.classList.contains('toast-err');
          if (txt && newToast.style.display !== 'none') {
            window.thrallShowToast(txt, isErr, true);
          } else {
            window.thrallShowToast('', false, false);
          }
        }
      }

      if (finalUrl) {
        try {
          var u = new URL(finalUrl, window.location.origin);
          u.searchParams.delete('ok');
          u.searchParams.delete('err');
          var cleanHref = u.pathname + (u.search ? u.search : '');
          if (pushHistory) {
            window.history.pushState(null, '', cleanHref);
          } else {
            window.history.replaceState(null, '', cleanHref);
          }
          syncActiveTabFromHref(cleanHref);
        } catch (e) {}
      }

      if (window.thrallRestoreTreeState) {
        window.thrallRestoreTreeState();
      }
    }

    function spaNavigate(href, pushHistory, keepToast) {
      syncActiveTabFromHref(href);
      recordBrowserBytes(String(href || '').length);
      return fetch(href, { credentials: 'same-origin' })
        .then(function (r) {
          var finalUrl = r.url || href;
          return r.text().then(function (txt) {
            recordBrowserBytes((txt && txt.length) || 0);
            applyParsedDoc(txt, finalUrl, pushHistory, keepToast);
          });
        })
        .catch(function () {
          window.location.href = href;
        });
    }
    window.thrallSpaNavigate = spaNavigate;

    document.addEventListener('click', function (e) {
      if (e.defaultPrevented || e.button !== 0 || e.metaKey || e.ctrlKey || e.shiftKey || e.altKey) return;
      var t = e.target;
      if (!t || typeof t.closest !== 'function') return;
      var link = t.closest('a[href]');
      if (!link) return;
      var href = link.getAttribute('href') || '';
      if (href.indexOf('/files?') === 0 && link.classList.contains('btn-secondary') && typeof window.URL === 'function' && typeof URL.createObjectURL === 'function') {
        e.preventDefault();
        recordBrowserBytes(href.length);
        fetch(href, { credentials: 'same-origin' })
          .then(function (r) {
            if (!r.ok) throw new Error('Download failed');
            return r.blob();
          })
          .then(function (blob) {
            recordBrowserBytes((blob && blob.size) || 0);
            var u = new URL(href, window.location.origin);
            var subParam = u.searchParams.get('sub') || 'download';
            var parts = subParam.split('/');
            var fname = parts[parts.length - 1] || 'download';
            var objUrl = URL.createObjectURL(blob);
            var a = document.createElement('a');
            a.href = objUrl;
            a.download = fname;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            setTimeout(function () { URL.revokeObjectURL(objUrl); }, 2000);
          })
          .catch(function () {
            window.location.href = href;
          });
        return;
      }
      if (link.getAttribute('target') === '_blank' || link.hasAttribute('download')) return;
      if (href.indexOf('/?') !== 0 && href !== '/') return;
      if (!document.querySelector('#middle-scroll .container')) return;
      e.preventDefault();
      spaNavigate(href, true, false);
    });

    document.addEventListener('submit', function (e) {
      if (e.defaultPrevented) return;
      var form = e.target;
      if (!form || !form.tagName || form.tagName.toUpperCase() !== 'FORM') return;
      var action = form.getAttribute('action') || '';
      if (!action || action === '/login' || action === '/logout') return;
      if (!document.querySelector('#middle-scroll .container')) return;
      e.preventDefault();
      var backdrop = document.getElementById('thrall-modal-backdrop');
      if (backdrop && backdrop.parentNode) backdrop.parentNode.removeChild(backdrop);
      var bodyParams = new URLSearchParams(new FormData(form)).toString();
      recordBrowserBytes(bodyParams.length);
      fetch(action, {
        method: 'POST',
        headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        body: bodyParams,
        credentials: 'same-origin'
      })
        .then(function (r) {
          var finalUrl = r.url || window.location.href;
          return r.text().then(function (txt) {
            recordBrowserBytes((txt && txt.length) || 0);
            applyParsedDoc(txt, finalUrl, true, false);
          });
        })
        .catch(function () {
          form.submit();
        });
    });

    window.addEventListener('popstate', function () {
      if (!document.querySelector('#middle-scroll .container')) return;
      spaNavigate(window.location.pathname + window.location.search, false, false);
    });
  }

  function start() {
    initPalette();
    renderHeaderWordmark();
    initCanvas();
    initSectionNav();
    initTelemetryPolling();
    initExplorerTree();
    initExplorerModals();
    initUploadEngine();
    initSpaNav();
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', start);
  } else {
    start();
  }
})();
