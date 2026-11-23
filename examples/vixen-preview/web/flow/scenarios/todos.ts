import type { Scenarios, Step } from '../flow.ts';

export const startLine = 17;

// Inclusive, 1-based Unicode coordinates in src/todos.rs, including imports.
// target/swap select returned fragments to explain; they never control swapping.
const addRequest = [
  { range: '354:4-354:16', to: 'top', highlight: false, button: 'add_todo_form()' },
  {
    range: [
      '356:25-356:41', // AddTodo::action()
      '357:25-357:44', // AddTodo::FIELD.title
    ],
    duration: 1000,
  },
  { range: '77:20-77:27', to: 'top', button: 'add()' },
] satisfies readonly Step[];

const searchRequest = [
  { range: '379:4-379:14', to: 'top', highlight: false, button: 'search_form()' },
  {
    range: [
      '380:18-383:36', // SearchTodos::action().hx().trigger(...).sync(...)
      '387:39-387:58', // SearchTodos::FIELD.q
      '393:50-393:74', // SearchTodos::FIELD.filter
    ],
    duration: 1000,
  },
  { range: '63:10-63:15', to: 'top', highlight: false, button: 'search()' },
] satisfies readonly Step[];

const toggleRequest = [
  { range: '320:4-320:12', to: 'top', highlight: false, button: 'todo_item()' },
  { range: '325:32-325:63', duration: 1000 }, // ToggleTodo::action().id(todo.id)
  { range: '107:10-107:15', to: 'top', highlight: false, button: 'toggle()' },
] satisfies readonly Step[];

const editRequest = [
  { range: '320:4-320:12', to: 'top', highlight: false, button: 'todo_item()' },
  { range: '330:28-330:57', duration: 1000 }, // EditTodo::action().id(todo.id)
  { range: '130:10-130:13', to: 'top', highlight: false, button: 'edit()' }
] satisfies readonly Step[];

const saveRequest = [
  { range: '336:4-336:22', to: 'top', highlight: false, button: 'todo_item_edit_form()' },
  {
    range: [
      '339:29-339:58', // SaveTodo::action().id(todo.id)
      '340:45-340:64', // SaveTodo::FIELD.done
      '342:29-342:49', // SaveTodo::FIELD.title
      '344:34-344:57', // SaveTodo::FIELD.priority
    ],
    duration: 1000,
  },
  { range: '150:10-150:13', to: 'top', highlight: false, button: 'save()' },
] satisfies readonly Step[];

const cancelRequest = [
  { range: '336:4-336:22', to: 'top', highlight: false, button: 'todo_item_edit_form()' },
  { range: '347:32-347:63', duration: 1000 }, // CancelEdit::action().id(todo.id)
  { range: '180:10-180:15', to: 'top', highlight: false, button: 'cancel()' },
] satisfies readonly Step[];

export const scenarios = {
  '/todos/search': {
    200: [
      ...searchRequest,
      { range: '67:14-67:51', target: '#todo-list' }, // todo_list(...)
    ],
  },
  '/todos/add': {
    200: [
      ...addRequest,
      {
        range: ['89:9-89:23', '89:26-89:49'],
        target: '#add-todo-form, #todo-count, #message',
      }, // add_todo_form(), todo_count(&state.todos),
      {
        range: ['92:17-92:71', '93:17-93:40'],
        target: 'li, #message[role="status"]', swap: 'beforeend'
      }, // Part::before_end(...), success(...)
      { range: '96:24-96:67', target: '#message[role="alert"]' }, // warning(...)
    ],
    422: [
      ...addRequest,
      { range: '81:13-81:44', target: '#message' }, // warning(...)
    ],
  },
  '/todos/toggle': {
    200: [
      ...toggleRequest,
      { range: '116:13-116:35', target: 'li', swap: 'outerHTML' }, // todo_item(&todo).into()
      { range: '118:13-118:45', target: 'li', swap: 'delete' }, // Part::delete(...)
      { range: '120:9-120:32', target: '#todo-count' }, // todo_count(&state.todos)
    ],
    404: [
      ...toggleRequest,
      { range: '111:48-111:76', target: '#message' }, // warning(...)
    ],
  },
  '/todos/edit': {
    200: [
      ...editRequest,
      { range: '137:14-137:38', target: 'li' }, // todo_item_edit_form(todo)
    ],
    404: [
      ...editRequest,
      { range: '134:48-134:76', target: '#message' }, // warning(...)
    ],
  },
  '/todos/save': {
    200: [
      ...saveRequest,
      { range: '165:13-165:35', target: 'li label span', swap: 'outerHTML' }, // todo_item(&todo).into()
      { range: '167:13-167:45', target: 'li', swap: 'delete' }, // Part::delete(...)
      { range: '169:9-169:32', target: '#todo-count' }, // todo_count(&state.todos)
      { range: '170:9-170:32', target: '#message' }, // success(...)
    ],
    404: [
      ...saveRequest,
      { range: '160:48-160:76', target: '#message' }, // warning(...)
    ],
    422: [
      ...saveRequest,
      { range: '150:10-150:13', to: 'top', highlight: false },
      { range: '154:13-154:44', target: '#message' }, // warning(...)
    ],
  },
  '/todos/cancel': {
    200: [
      ...cancelRequest,
      { range: '187:14-187:28', target: 'li' }, // todo_item(todo)
    ],
    404: [
      ...cancelRequest,
      { range: '184:48-184:76', target: '#message' }, // warning(...)
    ],
  },
} satisfies Scenarios;
