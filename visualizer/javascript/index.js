document.getElementById('date').innerHTML = new Date().toDateString();

var data = [];
var paused = true;

class Action {
  constructor(sidecarId, reason) {
    this.sidecarId = sidecarId;
    this.reason = reason;
  }
}

class Match {
  constructor(instant) {
    this.instant = instant;
    this.actions = []
  }

  addAction(sidecarId, reason) {
    this.actions.push(new Action(sidecarId, reason))
  }
}

// Parse all lines that begin with quack_log and return an array of arrays
// [secondsSinceStart, sidecarId, reasonInt]. Possible reasons include:
// sent, quacked, acked, detect_lost_packets, sidecar_detect_lost_packets.
async function parseFile(file) {
  const text = (await file.text()).split('\n')
  const re = /quack_log Instant { tv_sec: (\d+), tv_nsec: (\d+) } (\d+) \((\S+)\).*/

  var minTime = null;
  const matches = text.map(function(line) {
    const match = re.exec(line)
    if (match) {
      const instant = parseInt(match[1]) + parseInt(match[2]) / 10**9;
      const sidecar_id = match[3];
      const reason = match[4];
      return [instant, sidecar_id, reason]
    } else {
      return null
    }
  }).filter(function(match) {
    return match;
  }).map(function(match) {
    if (!minTime)
      minTime = match[0]
    match[0] -= minTime;
    return match;
  })

  return matches;
}

// Group actions executed at the same time (within 0.000001s tolerance).
function combineActions(matches) {
  const combined = [new Match(matches[0][0])];
  var currMatch = combined[0];
  matches.forEach(function(match) {
    if (Math.abs(match[0] - currMatch.instant) > 0.000001) {
      currMatch = new Match(match[0]);
      combined.push(currMatch);
    }
    currMatch.addAction(match[1], match[2]);
  })
  return combined;
}

// Creates a span element for the given reason.
function createSpan(sidecarId) {
  const span = document.createElement("span");
  span.id = sidecarId;
  span.alt = sidecarId;
  span.innerHTML = "0";
  span.classList.add("box");
  span.classList.add("sent");
  return span;
}

function applyFrame(index) {
  if (index >= data.length) {
    console.error('index outside of frame length');
    return;
  }
  const frame = data[index];
  document.getElementById('timeSinceStart').innerHTML = frame.instant
  frame.actions.forEach(function(action) {
    if (action.reason == "sent") {
      const span = createSpan(action.sidecarId);
      const container = document.getElementById('container');
      container.appendChild(span);
      container.appendChild(document.createTextNode(' '))
    } else {
      document.getElementById(action.sidecarId).classList.add(action.reason)
    }
  })
}

function removeFrame(index) {
  const frame = data[index];
  const prevFrame = data[index - 1];
  document.getElementById('timeSinceStart').innerHTML = prevFrame.instant
  frame.actions.forEach(function(action) {
    if (action.reason == "sent") {
      document.getElementById(action.sidecarId).remove()
    } else {
      document.getElementById(action.sidecarId).classList.remove(action.reason)
    }
  })
}

document.getElementById('myFile').onchange = async function(a, b, c) {
  const matches = await parseFile(this.files[0]);
  data = combineActions(matches);
  document.getElementById('maxFrames').innerHTML = data.length - 1;
  document.getElementById('container').innerHTML = '';
  document.getElementById('frameNumber').value = 0;
  console.log(data);
  applyFrame(0);
}

function clickForward() {
  const frameInput = document.getElementById('frameNumber');
  const index = parseInt(frameInput.value) + 1;
  frameInput.value = index;
  applyFrame(index);
}
document.getElementById('forward-button').onclick = clickForward;

document.getElementById('backward-button').onclick = function() {
  const frameInput = document.getElementById('frameNumber');
  const index = parseInt(frameInput.value);
  if (index == 0)
    return;
  frameInput.value = index - 1;
  removeFrame(index);
}

document.getElementById('jump-button').onclick = function() {
  const frameInput = document.getElementById('frameNumber');
  const index = parseInt(frameInput.value);
  document.getElementById('container').innerHTML = '';
  for (let i = 0; i <= index; i++) {
    applyFrame(i)
  }
}

document.getElementById('play-button').onclick = async function() {
  paused = false;
  const maxFrame = parseInt(document.getElementById('maxFrames').innerHTML)
  const frameInput = document.getElementById('frameNumber');
  while (parseInt(frameInput.value) < maxFrame) {
    if (paused) {
      break;
    }
    const currFrame = parseInt(frameInput.value)
    const secsUntilNextFrame = data[currFrame+1].instant - data[currFrame].instant;
    await new Promise(r => setTimeout(r, Math.round(secsUntilNextFrame*1000)));
    frameInput.value = currFrame + 1;
    applyFrame(currFrame + 1);
  }
}

document.getElementById('pause-button').onclick = function() {
  paused = true;
}
