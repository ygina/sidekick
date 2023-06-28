document.getElementById('date').innerHTML = new Date().toDateString();

// Parse all lines that begin with quack_log and return an array of arrays
// [seconds_since_start, sidecar_id, reason]. Possible reasons include: sent,
// quacked, acked, detect_lost_packets, sidecar_detect_lost_packets.
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
    match[0] -= minTime
    return match;
  })

  return data;
}

document.getElementById('myFile').onchange = async function(a, b, c) {
  const data = await parseFile(this.files[0]);
  console.log(data);
}
