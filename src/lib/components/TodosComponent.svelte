<script lang="ts">
	import { Todo } from '$lib';
	import * as Card from '$lib/components/ui/card';
	import { Button } from './ui/button';
	import { X } from 'lucide-svelte';
	import { Textarea } from './ui/textarea';
	import { Status } from '$lib';
	import { Checkbox } from "$lib/components/ui/checkbox/index.js";

	let { todos } = $props();
</script>

{#snippet todo(todo: Todo)}
	<Card.Root class={todo.status === Status.Complete ? 'opacity-60' : ''}>
		<Card.Header class="relative">
			<div class="absolute top-2 left-2">
				<Checkbox
					class="w-4 h-4 rounded-sm border-primary"
					checked={todo.status === Status.Complete}
					onCheckedChange={(checked) => {
						todo.status = checked ? Status.Complete : Status.Incomplete;
						todo.update();
					}}
				/>
			</div>
			<Button
				class="absolute top-0 right-0 h-8 w-8 hover:text-destructive"
				tabindex={-1}
				variant="ghost"
				size="icon"
				onclick={() => todo.delete()}
			>
				<X class="h-4 w-4" />
			</Button>
			<Card.Title>
				<input
					class="w-full bg-transparent border-none focus:outline-none text-lg font-semibold ml-6 {todo.status ===
						'Complete' && 'line-through'}"
					bind:value={todo.title}
					placeholder="Enter todo title..."
					onchange={() => todo.update()}
				/>
			</Card.Title>
		</Card.Header>
		<Card.Content>
			<Textarea
				data-textarea={`${todo.id}`}
				spellcheck="false"
				bind:value={todo.description}
				placeholder="Enter todo description..."
				onchange={() => todo.update()}
				class={todo.status === 'Complete' ? 'line-through' : ''}
			/>
			{#if todo.error}
				<p class="text-red-500">{todo.error}</p>
			{/if}
		</Card.Content>
	</Card.Root>
{/snippet}

<div class="overflow-auto w-full h-full space-y-2 p-2">
	{#each todos as t (t.id)}
		{@render todo(t)}
	{/each}
</div>
