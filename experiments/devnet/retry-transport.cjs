// Local fault injection: forward requests, suppress one successful submit reply.
'use strict';
const http = require('node:http');
const path = require('node:path');

async function startDropProxy(sdkRoot) {
  const websocket = require(path.join(sdkRoot, 'node_modules/websocket'));
  const server = http.createServer();
  const wsServer = new websocket.server({ httpServer: server });
  const connections = new Set();
  const stats = { armed: false, droppedSubmitReplies: 0 };
  wsServer.on('request', request => {
    const downstream = request.accept(null, request.origin);
    connections.add(downstream);
    const client = new websocket.client();
    let upstream;
    const queued = [];
    function forward(connection, message) {
      if (message.type === 'utf8') connection.sendUTF(message.utf8Data);
      else connection.sendBytes(message.binaryData);
    }
    downstream.on('message', message => {
      if (upstream) forward(upstream, message);
      else queued.push(message);
    });
    downstream.on('close', () => { connections.delete(downstream); upstream?.close(); });
    client.on('connectFailed', () => downstream.close());
    client.on('connect', connection => {
      upstream = connection;
      connections.add(connection);
      for (const message of queued.splice(0)) forward(connection, message);
      connection.on('message', message => {
        if (stats.armed && message.type === 'utf8') {
          let reply;
          try { reply = JSON.parse(message.utf8Data); } catch { /* Forward unparsed messages. */ }
          if (reply?.method === 'submitTransaction' && reply.params?.transactionId && !reply.error) {
            stats.armed = false;
            stats.droppedSubmitReplies++;
            return;
          }
        }
        if (downstream.connected) forward(downstream, message);
      });
      connection.on('close', () => { connections.delete(connection); downstream.close(); });
      connection.on('error', () => downstream.close());
    });
    downstream.on('error', () => upstream?.close());
    client.connect('ws://127.0.0.1:18310');
  });
  await new Promise((resolve, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', resolve);
  });
  return {
    url: `ws://127.0.0.1:${server.address().port}`,
    stats,
    arm() { stats.armed = true; },
    async close() {
      for (const connection of connections) connection.drop();
      wsServer.shutDown();
      await new Promise(resolve => server.close(resolve));
    },
  };
}
module.exports = { startDropProxy };
