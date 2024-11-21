<script lang="ts">
	import { GlobalState } from '$lib';
	import * as Resizable from '$lib/components/ui/resizable/index.js';
	import { Textarea } from '$lib/components/ui/textarea/index.js';
	import { Input } from '$lib/components/ui/input';
	import { once, emit } from '@tauri-apps/api/event';
	import TodosComponent from '$lib/components/TodosComponent.svelte';
	import { Loader2 } from 'lucide-svelte';

	let titleInput: HTMLInputElement | null = $state(null);
	let dbStatus: string = $state('loading');

	const gs = new GlobalState();

	$effect.pre(() => {
		const unlisten = once<{ status: string }>('dbstatus', (e) => {
			dbStatus = e.payload.status;
			if (e.payload.status === 'ready') {
				gs.fetchTodos();
				console.log('todos fetched');
			} else {
				console.error(e.payload.status);
			}
		});

		// Disable right click and F5 when not in dev in order to prevent access to refresh, use shadcn-svelte context menu instead
		if (process.env.NODE_ENV !== 'development') {
			document.addEventListener('contextmenu', (e) => {
				e.preventDefault();
			});

			window.addEventListener('keydown', (e) => {
				if (e.key === 'F5' || (e.ctrlKey && e.key === 'r')) {
					e.preventDefault();
				}
			});
		}

		emit('front-ready');

		return () => {
			unlisten.then((unlisten) => unlisten());
		};
	});
</script>

<div class="w-screen h-screen">
	<div class="container mx-auto h-full">
		{#if dbStatus === 'ready'}
			<Resizable.PaneGroup direction="vertical">
				<Resizable.Pane>
					<TodosComponent todos={gs.todos} />
				</Resizable.Pane>
				<Resizable.Handle withHandle />
				<Resizable.Pane maxSize={50} minSize={15} defaultSize={30} class="flex flex-col gap-2 p-4">
					<Input
						spellcheck="false"
						type="text"
						class="w-full"
						placeholder="Optionally enter todo title..."
						bind:ref={titleInput}
					/>
					<Textarea
						spellcheck="false"
						class="resize-none h-full"
						placeholder="Enter todo description and press enter (Shift+Enter for new line)..."
						onkeydown={(e) => {
							if (!titleInput) return;
							if (e.key === 'Enter' && !e.shiftKey) {
								if (titleInput.value.trim() === '') {
									const now = new Date();
									titleInput.value = now.toLocaleString(navigator.language, {
										year: '2-digit',
										month: 'short',
										day: '2-digit',
										hour: '2-digit',
										minute: '2-digit'
									});
								}
								gs.newTodo(titleInput.value, e.currentTarget.value);
								titleInput.value = '';
								e.currentTarget.value = '';
							}
						}}
					/>
					{#if gs.error}
						<p class="text-red-500">{gs.error}</p>
					{/if}
				</Resizable.Pane>
			</Resizable.PaneGroup>
		{:else if dbStatus === 'loading'}
			<div class="h-full flex items-center justify-center">
				<Loader2 class="h-8 w-8 animate-spin mx-auto text-gray-500" />
			</div>
		{:else}
			<div class="h-full flex items-center justify-center">
				<div class="text-center space-y-4">
					<h2 class="text-2xl font-semibold text-red-500">Database Error</h2>
					<p class="text-gray-600">Unable to connect to the database. Status: {dbStatus}</p>
					<!-- <Button variant="outline" onclick={() => window.location.reload()}>Retry</Button> -->
				</div>
			</div>
		{/if}
	</div>
</div>
