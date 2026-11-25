import type { Scenarios, Step } from '../flow.ts';

export const startLine = 17;

// Inclusive, 1-based Unicode coordinates in src/todos.rs, including imports.
// target/swap select returned fragments to explain; they never control swapping.
const addRequest = [
  { range: '71:8-71:14', to: 'top', highlight: false, button: 'AddTodo', skip: true },
  { range: '357:4-357:16', to: 'top', highlight: false, button: 'add_todo_form()' },
  {
    range: [
      '359:25-359:41', // AddTodo::action()
      '360:25-360:44', // AddTodo::FIELD.title
    ],
    duration: 1000,
  },
  { range: '76:20-76:26', to: 'top', button: 'add()', highlight: false },
] satisfies readonly Step[];

const searchRequest = [
  { range: '48:8-48:18', to: 'top', highlight: false, button: 'SearchTodos', skip: true },
  { range: '382:4-382:14', to: 'top', highlight: false, button: 'search_form()' },
  {
    range: [
      '383:18-383:38', // SearchTodos::action()
      '384:9-384:13', // .hx()
      '385:9-385:45', // .trigger(...)
      '386:9-386:36', // .sync(...)
      '390:39-390:58', // SearchTodos::FIELD.q
      '396:50-396:74', // SearchTodos::FIELD.filter
    ],
    duration: 1000,
  },
  { range: '62:25-62:35', to: 'top', button: 'search()', highlight: false },
] satisfies readonly Step[];

const toggleRequest = [
  { range: '102:8-102:17', to: 'top', highlight: false, button: 'ToggleTodo', skip: true },
  { range: '315:4-315:12', to: 'top', highlight: false, button: 'todo_item()' },
  { range: '320:32-320:63', duration: 1000 }, // ToggleTodo::action().id(todo.id)
  { range: '106:25-106:34', to: 'top', button: 'toggle()', highlight: false },
] satisfies readonly Step[];

const editRequest = [
  { range: '125:8-125:15', to: 'top', highlight: false, button: 'EditTodo', skip: true },
  { range: '315:4-315:12', to: 'top', highlight: false, button: 'todo_item()' },
  { range: '325:28-325:57', duration: 1000 }, // EditTodo::action().id(todo.id)
  { range: '129:32-129:39', to: 'top', button: 'edit()', highlight: false },
] satisfies readonly Step[];

const saveRequest = [
  { range: '141:8-141:15', to: 'top', highlight: false, button: 'SaveTodo', skip: true },
  { range: '331:4-331:22', to: 'top', highlight: false, button: 'todo_item_edit_form()' },
  {
    range: [
      '335:28-335:57', // SaveTodo::action().id(todo.id)
      '338:33-338:53', // SaveTodo::FIELD.done
      '340:33-340:53', // SaveTodo::FIELD.title
      '344:38-344:61', // SaveTodo::FIELD.priority
    ],
    duration: 1000,
  },
  { range: '149:21-149:28', to: 'top', button: 'save()', highlight: false },
] satisfies readonly Step[];

const cancelRequest = [
  { range: '175:8-175:17', to: 'top', highlight: false, button: 'CancelEdit', skip: true },
  { range: '331:4-331:22', to: 'top', highlight: false, button: 'todo_item_edit_form()' },
  { range: '348:40-348:71', duration: 1000 }, // CancelEdit::action().id(todo.id)
  { range: '179:36-179:45', to: 'top', button: 'cancel()', highlight: false },
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
        range: [
          '88:9-88:23', // add_todo_form()
          '88:26-88:49', // todo_count(&state.todos)
          '91:17-91:71', // Part::before_end(...)
          '92:17-92:40', // success(...)
        ],
        target: '#add-todo-form, #todo-count, #message, li',
      },
    ],
    422: [
      ...addRequest,
      { range: '80:13-80:44', target: '#message' }, // warning(...)
    ],
  },
  '/todos/toggle': {
    200: [
      ...toggleRequest,
      {
        range: [
          '115:13-115:35', // todo_item(&todo).into()
          '117:13-117:45', // Part::delete(...)
          '119:9-119:32', // todo_count(&state.todos)
        ],
        target: 'li, #todo-count',
      },
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
      {
        range: [
          '164:13-164:35', // todo_item(&todo).into()
          '166:13-166:45', // Part::delete(...)
          '168:9-168:32', // todo_count(&state.todos)
          '169:9-169:32', // success(...)
        ],
        target: 'li, #todo-count, #message',
      },
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
