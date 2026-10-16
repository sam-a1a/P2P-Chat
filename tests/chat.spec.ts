import { test, expect } from '@playwright/test';
import { spawn, type ChildProcessWithoutNullStreams } from 'node:child_process';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
let node: ChildProcessWithoutNullStreams;
let directory: string;
let output = '';
test.beforeAll(async () => {
  directory = await mkdtemp(join(tmpdir(), 'p2p-e2e-'));
  node = spawn(resolve('target/debug/p2p-chat-cli'), ['--identity', join(directory, 'identity.key'), '--tcp', '/ip4/127.0.0.1/tcp/0', '--websocket', '/ip4/127.0.0.1/tcp/19090/ws']);
  node.stdout.on('data', data => output += data.toString());
  node.stderr.on('data', data => output += data.toString());
  await expect.poll(() => output, { timeout: 15000 }).toContain('/tcp/19090/ws');
});
test.afterAll(async () => {
  node?.stdin.write('quit\n');
  if (node && node.exitCode === null) await new Promise<void>(resolve => { node.once('exit', () => resolve()); setTimeout(() => { node.kill(); resolve(); }, 3000); });
  if (directory) await rm(directory, { recursive: true, force: true });
});
for (const [name, port] of [['React', 5173], ['Solid', 5174]] as const) {
  test(`${name}: invalid address, messaging, disconnect and reconnect`, async ({ page }) => {
    await page.goto(`http://127.0.0.1:${port}`);
    await expect(page.getByRole('button', { name: 'Send', exact: true })).toBeDisabled();
    await page.getByLabel('Peer address').fill('invalid');
    await page.getByRole('button', { name: 'Connect', exact: true }).click();
    await expect(page.locator('.error')).toBeVisible();
    await expect(page.getByRole('status')).toHaveText('Status: disconnected');
    await page.getByLabel('Peer address').fill('/ip4/127.0.0.1/tcp/19090/ws');
    await page.getByRole('button', { name: 'Connect', exact: true }).click();
    await expect(page.getByRole('status')).toHaveText('Status: connected', { timeout: 20000 });
    // Wait for the subscription handshake before testing publish acknowledgement.
    await page.waitForTimeout(1500);
    const message = `${name} مرحبا ${Date.now()}`;
    await page.getByLabel('Message', { exact: true }).fill(message);
    await page.getByRole('button', { name: 'Send', exact: true }).click();
    await expect(page.getByRole('log')).toContainText(`You: ${message}`);
    await expect.poll(() => output).toContain(message);
    const reply = `Desktop to ${name} ${Date.now()}`;
    node.stdin.write(reply + '\n');
    await expect(page.getByRole('log')).toContainText(reply);
    await page.getByRole('button', { name: 'Disconnect', exact: true }).click();
    await expect(page.getByRole('status')).toHaveText('Status: disconnected');
    await expect(page.getByRole('button', { name: 'Send', exact: true })).toBeDisabled();
    await page.getByRole('button', { name: 'Connect', exact: true }).click();
    await expect(page.getByRole('status')).toHaveText('Status: connected');
    await page.waitForTimeout(1500);
    const reconnect = `Reconnected ${name} ${Date.now()}`;
    node.stdin.write(reconnect + '\n');
    await expect(page.getByRole('log')).toContainText(reconnect);
  });
}
test('React and Solid exchange messages through a desktop peer', async ({ browser }) => {
  const a = await browser.newPage(); const b = await browser.newPage();
  for (const [page, port] of [[a, 5173], [b, 5174]] as const) {
    await page.goto(`http://127.0.0.1:${port}`);
    await page.getByLabel('Peer address').fill('/ip4/127.0.0.1/tcp/19090/ws');
    await page.getByRole('button', { name: 'Connect', exact: true }).click();
    await expect(page.getByRole('status')).toHaveText('Status: connected');
  }
  await a.waitForTimeout(2000);
  for (const [sender, receiver, text] of [[a, b, 'React to Solid'], [b, a, 'Solid to React']] as const) {
    await sender.getByLabel('Message', { exact: true }).fill(text);
    await sender.getByRole('button', { name: 'Send', exact: true }).click();
    await expect(receiver.getByRole('log')).toContainText(text);
  }
  await a.close(); await b.close();
});
