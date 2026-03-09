import { writable, derived } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export interface Tag {
  id: number;
  name: string;
  color: string;
  repo_count: number;
}

// Stores
export const allTags = writable<Tag[]>([]);
export const selectedTagIds = writable<Set<number>>(new Set());
export const tagLoading = writable(false);

// Derived: Current tag filters as an array of names for easy comparison
export const activeTagFilters = derived(
  [allTags, selectedTagIds],
  ([$allTags, $selectedTagIds]) => {
    return $allTags
      .filter((t) => $selectedTagIds.has(t.id))
      .map((t) => t.name);
  }
);

// Actions
export async function loadTags() {
  tagLoading.set(true);
  try {
    const tags = await invoke<Tag[]>('list_tags');
    allTags.set(tags);
  } catch (e) {
    console.error('Failed to load tags:', e);
  } finally {
    tagLoading.set(false);
  }
}

export async function createTag(name: string, color: string) {
  try {
    const newTag = await invoke<Tag>('create_tag', { name, color });
    await loadTags(); // Refresh list to get counts and updated sorting
    return newTag;
  } catch (e) {
    throw e;
  }
}

export async function renameTag(id: number, newName: string) {
  try {
    await invoke('rename_tag', { id, newName });
    await loadTags();
  } catch (e) {
    throw e;
  }
}

export async function deleteTag(id: number) {
  try {
    await invoke('delete_tag', { id });
    selectedTagIds.update((ids) => {
      ids.delete(id);
      return new Set(ids);
    });
    await loadTags();
  } catch (e) {
    throw e;
  }
}

export async function assignTag(repoPath: string, tagId: number) {
  try {
    await invoke('assign_tag', { repoPath, tagId });
    await loadTags();
  } catch (e) {
    throw e;
  }
}

export async function removeTag(repoPath: string, tagId: number) {
  try {
    await invoke('remove_tag', { repoPath, tagId });
    await loadTags();
  } catch (e) {
    throw e;
  }
}

export function toggleTagFilter(tagId: number) {
  selectedTagIds.update((ids) => {
    if (ids.has(tagId)) {
      ids.delete(tagId);
    } else {
      ids.add(tagId);
    }
    return new Set(ids); // Trigger reactivity
  });
}

export function clearTagFilters() {
  selectedTagIds.set(new Set());
}

// Auto-refresh on scan completion
if (typeof window !== 'undefined') {
  listen('scan-completed', () => {
    loadTags();
  });
}
