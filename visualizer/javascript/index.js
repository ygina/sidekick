document.getElementById('date').innerHTML = new Date().toDateString();

const EPSILON = 0.001;
var data = [];
var paused = true;

class Action {
  constructor(sidecarId, reason) {
    this.sidecarId = sidecarId;
    this.reason = reason;
  }
}

class Match {
  constructor(match) {
    this.instant = match[0];
    this.cwnd = match[3] / 1500;
    this.pktsInFlight = match[4] / 1500;
    this.actions = []
  }

  addAction(sidecarId, reason) {
    this.actions.push(new Action(sidecarId, reason))
  }
}

function approxEqual(val1, val2) {
  return Math.abs(val1 - val2) < EPSILON;
}

function setInstantData(instant, cwnd, pktsInFlight) {
  document.getElementById('timeSinceStart').innerHTML = instant.toFixed(3)
  document.getElementById('congestionWindow').innerHTML = cwnd.toFixed(3)
  document.getElementById('packetsInFlight').innerHTML = pktsInFlight.toFixed(3)
}

function parseTextKey(text, key, minTime) {
  const re = new RegExp(key + " (\\d+) Instant { tv_sec: (\\d+), tv_nsec: (\\d+) }.*")
  return text.map(function(line) {
    const match = re.exec(line);
    if (match) {
      const instant = parseInt(match[2]) + parseInt(match[3]) / 10**9;
      const value = parseInt(match[1])
      return [instant - minTime, value]
    } else {
      return null
    }
  }).filter(function(match) {
    return match;
  })
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

  // Parse the congestion windows and bytes in flight. Consolidate the values
  // with the matches above..
  const cwnds = parseTextKey(text, "cwnd", minTime);
  const inFlight = parseTextKey(text, "bytes_in_flight", minTime);
  console.log(cwnds)
  console.log(inFlight)
  var cwndIndex = 0;
  var inFlightIndex = 0;
  return matches.map(function(match) {
    // Set currIndex to the smallest index such that the time of the next
    // cwnd is larger than the current time.
    const myTime = match[0];
    while (cwndIndex < cwnds.length - 1) {
      const nextTime = cwnds[cwndIndex + 1][0]
      if (nextTime < myTime || approxEqual(nextTime, myTime)) {
        cwndIndex += 1;
      } else {
        break;
      }
    }
    while (inFlightIndex < inFlight.length - 1) {
      const nextTime = inFlight[inFlightIndex + 1][0]
      if (nextTime < myTime || approxEqual(nextTime, myTime)) {
        inFlightIndex += 1;
      } else {
        break;
      }
    }
    return [myTime, match[1], match[2], cwnds[cwndIndex][1], inFlight[inFlightIndex][1]]
  })
}

// Group actions executed at the same time (within EPSILON seconds tolerance).
function combineActions(matches) {
  const combined = [new Match(matches[0])];
  var currMatch = combined[0];
  matches.forEach(function(match) {
    if (!approxEqual(match[0], currMatch.instant)) {
      currMatch = new Match(match);
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
  span.innerHTML = "&#9733;";
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
  setInstantData(frame.instant, frame.cwnd, frame.pktsInFlight)
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
  setInstantData(prevFrame.instant, prevFrame.cwnd, prevFrame.pktsInFlight)
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

document.getElementById('jumpToTime-button').onclick = function() {
  const frameInput = document.getElementById('frameNumber');
  const jumpInput = document.getElementById('jumpToTime');
  const time = parseFloat(jumpInput.value);
  document.getElementById('container').innerHTML = '';
  for (let index = 0; index < data.length; index++) {
    if (data[index].instant > time)
      break;
    applyFrame(index)
    frameInput.value = index
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
    await new Promise(r => setTimeout(r, Math.round(secsUntilNextFrame*10000)));
    frameInput.value = currFrame + 1;
    applyFrame(currFrame + 1);
  }
}

document.getElementById('pause-button').onclick = function() {
  paused = true;
}
