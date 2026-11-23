import type { Scenarios, Step } from '../flow.ts';

export const startLine = 17;

// Inclusive, 1-based Unicode coordinates in src/todos.rs, including imports.
// target/swap select returned fragments to explain; they never control swapping.
const addRequest = [
  { range: '71:8-71:14', to: 'top', highlight: false, button: 'AddTodo', skip: true },
  { range: '353:4-353:16', to: 'top', highlight: false, button: 'add_todo_form()' },
  {
    range: [
      '355:25-355:41', // AddTodo::action()
      '356:25-356:44', // AddTodo::FIELD.title
    ],
    duration: 1000,
  },
  { range: '76:20-76:26', to: 'top', button: 'add()' },
] satisfies readonly Step[];

const searchRequest = [
  { range: '48:8-48:18', to: 'top', highlight: false, button: 'SearchTodos', skip: true },
  { range: '378:4-378:14', to: 'top', highlight: false, button: 'search_form()' },
  {
    range: [
      '379:18-382:36', // SearchTodos::action().hx().trigger(...).sync(...)
      '386:39-386:58', // SearchTodos::FIELD.q
      '392:50-392:74', // SearchTodos::FIELD.filter
    ],
    duration: 1000,
  },
  { range: '62:25-62:35', to: 'top', button: 'search()' },
] satisfies readonly Step[];

const toggleRequest = [
  { range: '102:8-102:17', to: 'top', highlight: false, button: 'ToggleTodo', skip: true },
  { range: '319:4-319:12', to: 'top', highlight: false, button: 'todo_item()' },
  { range: '324:32-324:63', duration: 1000 }, // ToggleTodo::action().id(todo.id)
  { range: '106:25-106:34', to: 'top', button: 'toggle()' },
] satisfies readonly Step[];

const editRequest = [
  { range: '125:8-125:15', to: 'top', highlight: false, button: 'EditTodo', skip: true },
  { range: '319:4-319:12', to: 'top', highlight: false, button: 'todo_item()' },
  { range: '329:28-329:57', duration: 1000 }, // EditTodo::action().id(todo.id)
  { range: '129:32-129:39', to: 'top', button: 'edit()' }
] satisfies readonly Step[];

const saveRequest = [
  { range: '141:8-141:15', to: 'top', highlight: false, button: 'SaveTodo', skip: true },
  { range: '335:4-335:22', to: 'top', highlight: false, button: 'todo_item_edit_form()' },
  {
    range: [
      '338:29-338:58', // SaveTodo::action().id(todo.id)
      '339:45-339:64', // SaveTodo::FIELD.done
      '341:29-341:49', // SaveTodo::FIELD.title
      '343:34-343:57', // SaveTodo::FIELD.priority
    ],
    duration: 1000,
  },
  { range: '149:21-149:28', to: 'top', button: 'save()' },
] satisfies readonly Step[];

const cancelRequest = [
  { range: '175:8-175:17', to: 'top', highlight: false, button: 'CancelEdit', skip: true },
  { range: '335:4-335:22', to: 'top', highlight: false, button: 'todo_item_edit_form()' },
  { range: '346:32-346:63', duration: 1000 }, // CancelEdit::action().id(todo.id)
  { range: '179:36-179:45', to: 'top', button: 'cancel()' },
] satisfies readonly Step[];

export const scenarios = {
  '/todos/search': {
    200: [
      ...searchRequest,
      { range: '66:14-66:51', target: '#todo-list' }, // todo_list(...)
    ],
  },
  '/todos/add': {
    200: [
      ...addRequest,
      {
        range: ['88:9-88:23', '88:26-88:49'],
        target: '#add-todo-form, #todo-count, #message',
      }, // add_todo_form(), todo_count(&state.todos),
      {
        range: ['91:17-91:71', '92:17-92:40'],
        target: 'li, #message[role="status"]', swap: 'beforeend'
      }, // Part::before_end(...), success(...)
      { range: '95:24-95:67', target: '#message[role="alert"]' }, // warning(...)
    ],
    422: [
      ...addRequest,
      { range: '80:13-80:44', target: '#message' }, // warning(...)
    ],
  },
  '/todos/toggle': {
    200: [
      ...toggleRequest,
      { range: '115:13-115:35', target: 'li', swap: 'outerHTML' }, // todo_item(&todo).into()
      { range: '117:13-117:45', target: 'li', swap: 'delete' }, // Part::delete(...)
      { range: '119:9-119:32', target: '#todo-count' }, // todo_count(&state.todos)
    ],
    404: [
      ...toggleRequest,
      { range: '110:48-110:76', target: '#message' }, // warning(...)
    ],
  },
  '/todos/edit': {
    200: [
      ...editRequest,
      { range: '136:14-136:38', target: 'li' }, // todo_item_edit_form(todo)
    ],
    404: [
      ...editRequest,
      { range: '133:48-133:76', target: '#message' }, // warning(...)
    ],
  },
  '/todos/save': {
    200: [
      ...saveRequest,
      { range: '164:13-164:35', target: 'li label span', swap: 'outerHTML' }, // todo_item(&todo).into()
      { range: '166:13-166:45', target: 'li', swap: 'delete' }, // Part::delete(...)
      { range: '168:9-168:32', target: '#todo-count' }, // todo_count(&state.todos)
      { range: '169:9-169:32', target: '#message' }, // success(...)
    ],
    404: [
      ...saveRequest,
      { range: '159:48-159:76', target: '#message' }, // warning(...)
    ],
    422: [
      ...saveRequest,
      { range: '153:13-153:44', target: '#message' }, // warning(...)
    ],
  },
  '/todos/cancel': {
    200: [
      ...cancelRequest,
      { range: '186:14-186:28', target: 'li' }, // todo_item(todo)
    ],
    404: [
      ...cancelRequest,
      { range: '183:48-183:76', target: '#message' }, // warning(...)
    ],
  },
} satisfies Scenarios;
