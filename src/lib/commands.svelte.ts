import { invoke } from '@tauri-apps/api/core';

export const enum Status {
	Incomplete = 'Incomplete',
	Complete = 'Complete'
}

interface Data {
	id: number;
	title: string;
	description: string;
	status: Status;
}

type ErrorResponse = {
	0: string;
	1: Data | null;
};

// Utility Functions
export const preventDefault = <T extends Event>(fn: (e: T) => void): ((e: T) => void) => {
	return (e: T) => {
		e.preventDefault();
		fn(e);
	};
};

const catchError = async <T>(p: Promise<T>): Promise<[undefined, T] | [Error]> => {
	try {
		const r = await p;
		return [undefined, r] as [undefined, T];
	} catch (e) {
		return [e] as [Error];
	}
};

// Models
export class Todo {
	private _state = $state({
		id: 0,
		title: '',
		description: '',
		status: Status.Incomplete
	});
	private _error? = $state<string | undefined>(undefined);
	private _onDelete: (id: number) => void;

	constructor(
		id: number,
		title: string,
		description: string,
		status: Status = Status.Incomplete,
		onDelete: (id: number) => void
	) {
		this._state.id = id;
		this._state.title = title;
		this._state.description = description;
		this._state.status = status;
		this._onDelete = onDelete;
	}

	get id() {
		return this._state.id;
	}

	get title() {
		return this._state.title;
	}

	set title(value: string) {
		this._state.title = value;
	}

	get description() {
		return this._state.description;
	}

	set description(value: string) {
		this._state.description = value;
	}

	get status() {
		return this._state.status;
	}

	set status(value: Status) {
		this._state.status = value;
	}

	get error() {
		return this._error;
	}

	async update() {
		const { id, title, description, status } = this._state;
		const [e, r] = await catchError(invoke('update_todo', { todo: { id, title, description, status } }));
		if (e) {
			// attempt to rollback to previous state
			const error = e as unknown as ErrorResponse;
			if (error[1] === null) {
				this._error = error[0];
				return;
			}
			this._error = error[0];
			this._state.title = error[1].title;
			this._state.description = error[1].description;
			this._state.status = error[1].status;
		} else {
			this._error = undefined;
			const updated = r as Data;
			this._state.title = updated.title;
			this._state.description = updated.description;
			this._state.status = updated.status;
		}
	}

	async delete() {
		const [e, _] = await catchError(invoke('delete_todo', { id: this._state.id }));
		if (e) {
			const error = e as unknown as ErrorResponse;
			if (error[1] === null) {
				this._error = error[0];
				return;
			}
			this._error = error[0];
			this._state.title = error[1].title;
			this._state.description = error[1].description;
			this._state.status = error[1].status;
		} else {
			this._error = undefined;
			this._onDelete(this._state.id);
		}
	}
}

export class GlobalState {
	private _state = $state({
		todos: [] as Todo[],
		error: undefined as string | undefined
	});

	get todos() {
		return this._state.todos;
	}

	get error() {
		return this._state.error;
	}

	private deleteTodo(id: number) {
		const i = this._state.todos.findIndex((t) => t.id === id);
		if (i !== -1) {
			this._state.todos.splice(i, 1);
		}
	}

	async newTodo(title: string, description: string) {
		const [e, t] = await catchError(invoke<Data>('add_todo', { title, description }));
		if (e) {
			this._state.error = e instanceof Error ? e.message : String(e);
		} else if (t) {
			this._state.todos.push(new Todo(t.id, t.title, t.description, t.status, (id) => this.deleteTodo(id)));
			this._state.error = undefined;
		}
	}

	async fetchTodos() {
		const [e, tds] = await catchError(invoke<Array<Data>>('get_todos'));
		if (e) {
			this._state.error = e instanceof Error ? e.message : String(e);
		} else if (tds) {
			this._state.todos = [];
			for (const t of tds) {
				this._state.todos.push(
					new Todo(t.id, t.title, t.description, t.status, (id) => this.deleteTodo(id))
				);
			}
			this._state.error = undefined;
		}
	}
}
