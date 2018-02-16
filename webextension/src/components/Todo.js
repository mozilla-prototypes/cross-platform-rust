import React, { Component } from 'react';
import PropTypes from 'prop-types';
import TodoDropdownMenu from './TodoDropdownMenu';
import DatesChooser from './DatesChooser';
import './Todo.css';

function toPrettyDate(date) {
  return new Date(date).toISOString().substr(0, 10);
}

class Todo extends Component {
  static propTypes = {
    onRemoveTodo: PropTypes.func.isRequired,
    onTodoNameChanged: PropTypes.func.isRequired,
    onTodoCompleted: PropTypes.func.isRequired,
    onTodoChangeCompletionDate: PropTypes.func.isRequired,
    name: PropTypes.string.isRequired,
    completionDate: PropTypes.number,
    uuid: PropTypes.string.isRequired
  }

  state = {
    datesDropdownOpen: false,
    isEditingName: false,
    newName: ''
  }

  onSubmit = (e) => {
    e.preventDefault();
    this.toggleEdit();
  }

  toggleEdit = () => {
    this.setState({isEditingName: !this.state.isEditingName});
    if (!this.state.isEditingName) {
      this.setState({newName: this.props.name});
    } else {
      this.props.onTodoNameChanged(this.props.uuid, this.state.newName);
    }
  }

  handleCompleted = (e) => {
    this.props.onTodoCompleted(this.props.uuid, e.target.checked);
  }

  handleKeyDown = (e) => {
    // Cancel edit on escape key
    if (e.keyCode === 27) {
      this.setState({isEditingName: !this.state.isEditingName});
    }
  }

  toggleDatesDropdown = () => {
    this.setState({datesDropdownOpen: !this.state.datesDropdownOpen});
  }

  handleNewNameChange = (event) => {
    this.setState({newName: event.target.value});
  }

  render() {
    const { uuid, name, completionDate, onRemoveTodo, onTodoChangeCompletionDate } = this.props;
    const { datesDropdownOpen, isEditingName } = this.state;

    const todoChangeCompletionDate = (date) => onTodoChangeCompletionDate(uuid, date);

    return (
      <div className="todo-wrapper">
        <div className={`todo${ datesDropdownOpen ? ' dropdown-open' : ''}`}>
          <input className="todo-completed" checked={!!completionDate} onChange={this.handleCompleted} type="checkbox"
            title={completionDate? `Completed on ${ toPrettyDate(completionDate)}` : ''} />
          <div className="todo-content">
            <div className="todo-details-wrapper">
              <div className="todo-details">
                <div className="todo-name">
                  {
                    !isEditingName ?
                      <div onDoubleClick={this.toggleEdit}>{name}</div> :
                      <form onSubmit={this.onSubmit}>
                        <input className="todo-edit-name" onKeyDown={this.handleKeyDown} value={this.state.newName} onChange={this.handleNewNameChange} />
                      </form>
                  }
                </div>
              </div>
              <div className="todo-buttons-wrapper">
                <div className="todo-dropdown-wrapper">
                  <div className="todo-dates-button" onClick={this.toggleDatesDropdown}>
                    <span role="img" aria-label="Dates">📅</span>
                  </div>
                  {datesDropdownOpen ?
                    <TodoDropdownMenu onCloseDropdown={this.toggleDatesDropdown}>
                      <DatesChooser completionDate={completionDate}
                        onTodoChangeCompletionDate={todoChangeCompletionDate} />
                    </TodoDropdownMenu> : null
                  }
                </div>
                <div className="todo-delete-button" onClick={onRemoveTodo}>
                  <span role="img" aria-label="Delete">❌</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    );
  }
}

export default Todo;
