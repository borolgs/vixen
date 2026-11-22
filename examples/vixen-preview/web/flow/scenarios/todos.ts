import type { Scenarios, Step } from '../flow.ts';

export const startLine = 12;

// Inclusive, 1-based Unicode coordinates in src/todos.rs, including imports.
// target/swap select returned fragments to explain; they never control swapping.
const addRequest = [
  { range: '342:4-342:16', to: 'top', highlight: false, button: 'add_todo_form()' },
  {
    range: [
      '344:25-344:41', // AddTodo::action()
      '345:25-345:44', // AddTodo::FIELD.title
    ],
    duration: 1000,
  },
  { range: '69:48-69:55', to: 'top', button: 'add()' },
] satisfies readonly Step[];

const searchRequest = [
  { range: '367:4-367:14', to: 'top', highlight: false, button: 'search_form()' },
  {
    range: [
      '368:18-371:36', // SearchTodos::action().hx().trigger(...).sync(...)
      '375:39-375:58', // SearchTodos::FIELD.q
      '381:50-381:74', // SearchTodos::FIELD.filter
    ],
    duration: 1000,
  },
  { range: '52:10-52:15', to: 'top', highlight: false, button: 'search()' },
] satisfies readonly Step[];

const toggleRequest = [
  { range: '308:4-308:12', to: 'top', highlight: false, button: 'todo_item()' },
  { range: '313:32-313:63', duration: 1000 }, // ToggleTodo::action().id(todo.id)
  { range: '99:10-99:15', to: 'top', highlight: false, button: 'toggle()' },
] satisfies readonly Step[];

const editRequest = [
  { range: '308:4-308:12', to: 'top', highlight: false, button: 'todo_item()' },
  { range: '318:28-318:57', duration: 1000 }, // EditTodo::action().id(todo.id)
  { range: '125:10-125:13', to: 'top', highlight: false, button: 'edit()' }
] satisfies readonly Step[];

const saveRequest = [
  { range: '324:4-324:22', to: 'top', highlight: false, button: 'todo_item_edit_form()' },
  {
    range: [
      '327:29-327:58', // SaveTodo::action().id(todo.id)
      '328:45-328:64', // SaveTodo::FIELD.done
      '330:29-330:49', // SaveTodo::FIELD.title
      '332:34-332:57', // SaveTodo::FIELD.priority
    ],
    duration: 1000,
  },
  { range: '148:10-148:13', to: 'top', highlight: false, button: 'save()' },
] satisfies readonly Step[];

const cancelRequest = [
  { range: '324:4-324:22', to: 'top', highlight: false, button: 'todo_item_edit_form()' },
  { range: '335:32-335:63', duration: 1000 }, // CancelEdit::action().id(todo.id)
  { range: '178:10-178:15', to: 'top', highlight: false, button: 'cancel()' },
] satisfies readonly Step[];

export const scenarios = {
  '/todos/search': {
    200: [
      ...searchRequest,
      { range: '59:14-59:51', target: '#todo-list' }, // todo_list(...)
    ],
  },
  '/todos/add': {
    200: [
      ...addRequest,
      {
        range: ['81:9-81:23', '81:26-81:49'],
        target: '#add-todo-form, #todo-count, #message',
      }, // add_todo_form(), todo_count(&state.todos),
      {
        range: ['84:17-84:71', '85:17-85:40'],
        target: 'li, #message[role="status"]', swap: 'beforeend'
      }, // Part::before_end(...), success(...)
      { range: '88:24-88:67', target: '#message[role="alert"]' }, // warning(...)
    ],
    422: [
      ...addRequest,
      { range: '73:13-73:44', target: '#message' }, // warning(...)
    ],
  },
  '/todos/toggle': {
    200: [
      ...toggleRequest,
      { range: '111:13-111:35', target: 'li', swap: 'outerHTML' }, // todo_item(&todo).into()
      { range: '113:13-113:45', target: 'li', swap: 'delete' }, // Part::delete(...)
      { range: '115:9-115:32', target: '#todo-count' }, // todo_count(&state.todos)
    ],
    404: [
      ...toggleRequest,
      { range: '106:48-106:76', target: '#message' }, // warning(...)
    ],
  },
  '/todos/edit': {
    200: [
      ...editRequest,
      { range: '135:14-135:38', target: 'li' }, // todo_item_edit_form(todo)
    ],
    404: [
      ...editRequest,
      { range: '132:48-132:76', target: '#message' }, // warning(...)
    ],
  },
  '/todos/save': {
    200: [
      ...saveRequest,
      { range: '163:13-163:35', target: 'li label span', swap: 'outerHTML' }, // todo_item(&todo).into()
      { range: '165:13-165:45', target: 'li', swap: 'delete' }, // Part::delete(...)
      { range: '167:9-167:32', target: '#todo-count' }, // todo_count(&state.todos)
      { range: '168:9-168:32', target: '#message' }, // success(...)
    ],
    404: [
      ...saveRequest,
      { range: '158:48-158:76', target: '#message' }, // warning(...)
    ],
    422: [
      ...saveRequest,
      { range: '148:10-148:13', to: 'top', highlight: false },
      { range: '152:13-152:44', target: '#message' }, // warning(...)
    ],
  },
  '/todos/cancel': {
    200: [
      ...cancelRequest,
      { range: '188:14-188:28', target: 'li' }, // todo_item(todo)
    ],
    404: [
      ...cancelRequest,
      { range: '185:48-185:76', target: '#message' }, // warning(...)
    ],
  },
} satisfies Scenarios;
