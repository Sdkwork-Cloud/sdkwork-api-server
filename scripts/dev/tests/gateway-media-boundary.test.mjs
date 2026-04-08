import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test from 'node:test';

const repoRoot = path.resolve(import.meta.dirname, '..', '..', '..');
const gatewayRoot = path.join(repoRoot, 'crates', 'sdkwork-api-app-gateway', 'src');

function readGatewaySource(fileName) {
  return readFileSync(path.join(gatewayRoot, fileName), 'utf8');
}

test('relay_files_uploads keeps music and video fallback helpers out of the files/uploads surface', () => {
  const relayFilesUploads = readGatewaySource('relay_files_uploads.rs');
  const relayMusicVideo = readGatewaySource('relay_music_video.rs');

  for (const signature of [
    'pub fn create_music(',
    'pub fn list_music(',
    'pub fn get_music(',
    'pub fn delete_music(',
    'pub fn music_content(',
    'pub fn create_music_lyrics(',
    'pub fn create_video(',
    'pub fn list_videos(',
    'pub fn get_video(',
    'pub fn delete_video(',
    'pub fn video_content(',
    'pub fn remix_video(',
    'pub fn create_video_character(',
    'pub fn list_video_characters(',
    'pub fn get_video_character(',
    'pub fn get_video_character_canonical(',
    'pub fn update_video_character(',
    'pub fn extend_video(',
    'pub fn edit_video(',
    'pub fn extensions_video(',
  ]) {
    assert.equal(
      relayFilesUploads.includes(signature),
      false,
      `relay_files_uploads.rs should not define legacy media fallback ${signature}`,
    );
    assert.equal(
      relayMusicVideo.includes(signature),
      true,
      `relay_music_video.rs should define canonical media fallback ${signature}`,
    );
  }
});
