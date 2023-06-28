document.getElementById('date').innerHTML = new Date().toDateString();

var data = [];

class Match {
  constructor(instant, sidecarId, reason) {
    this.instant = instant;
    this.sidecarId = sidecarId;
    this.reason = reason;
  }
}

// Parse all lines that begin with quack_log and return an array of arrays
// [secondsSinceStart, sidecarId, reasonInt]. Possible reasons include:
// sent, quacked, acked, detect_lost_packets, sidecar_detect_lost_packets.
async function parseFile(file) {
  const text = (await file.text()).split('\n')
  const re = /quack_log Instant { tv_sec: (\d+), tv_nsec: (\d+) } (\d+) \((\S+)\).*/

  var minTime = null;
  const data = text.map(function(line) {
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
    return new Match(match[0] - minTime, match[1], match[2]);
  })

  return data;
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
  if (frame.reason == "sent") {
    const span = createSpan(frame.sidecarId);
    const container = document.getElementById('container');
    container.appendChild(span);
    container.appendChild(document.createTextNode(' '))
  } else {
    document.getElementById(frame.sidecarId).classList.add(frame.reason)
  }
}

function removeFrame(index) {
  const frame = data[index];
  const prevFrame = data[index - 1];
  document.getElementById('timeSinceStart').innerHTML = prevFrame.instant
  if (frame.reason == "sent") {
    document.getElementById(frame.sidecarId).remove()
  } else {
    document.getElementById(frame.sidecarId).classList.remove(frame.reason)
  }
}

document.getElementById('myFile').onchange = async function(a, b, c) {
  data = await parseFile(this.files[0]);
  document.getElementById('maxFrames').innerHTML = data.length;
  document.getElementById('container').innerHTML = '';
  console.log(data);
  applyFrame(0);
}

document.getElementById('forward-button').onclick = function() {
  const frameInput = document.getElementById('frameNumber');
  const index = parseInt(frameInput.value) + 1;
  frameInput.value = index;
  applyFrame(index);
}

document.getElementById('backward-button').onclick = function() {
  const frameInput = document.getElementById('frameNumber');
  const index = parseInt(frameInput.value);
  if (index == 0)
    return;
  frameInput.value = index - 1;
  removeFrame(index);
}
