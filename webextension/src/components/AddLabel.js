import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { GithubPicker } from 'react-color';
import onClickOutside from 'react-onclickoutside';
import './AddLabel.css';

const ColorPicker = onClickOutside(class ColorPicker extends Component {

  static propTypes = {
    requestClose: PropTypes.func.isRequired,
    colorChange: PropTypes.func.isRequired,
  }

  handleColorChange = (color) => {
    this.props.colorChange(color);
    this.props.requestClose();
  }

  handleClickOutside = () => {
    this.props.requestClose();
  };

  render() {
    return <GithubPicker
      onChange={ this.handleColorChange }
    />;
  }
});

const DEFAULT_STATE = {displayColorPicker: false, newLabelName: '', newLabelColor: 'rgb(184, 0, 0)'};

class AddLabel extends Component {

  static propTypes = {
    addLabel: PropTypes.func.isRequired
  }

  state = DEFAULT_STATE

  handleNameChange = (event) => {
    this.setState({newLabelName: event.target.value});
  }

  handleColorClick = () => {
    this.setState({ displayColorPicker: !this.state.displayColorPicker });
  }

  handleColorChange = (color) => {
    this.setState({ newLabelColor: color.hex });
  }

  handleColorClose = () => {
    this.setState({ displayColorPicker: false });
  }

  onSubmit = (e) => {
    e.preventDefault();
    const newLabelName = this.state.newLabelName;
    if (!newLabelName.trim()) {
      return;
    }
    this.props.addLabel(newLabelName, this.state.newLabelColor);
    this.setState(DEFAULT_STATE);
  }

  render() {
    return (
      <form className="add-label-form" onSubmit={this.onSubmit}>
        <div className="color-swatch-wrapper"
          onClick={ this.handleColorClick }>
          <div style={{ backgroundColor: this.state.newLabelColor}}
            className="color-swatch" />
        </div>
        { this.state.displayColorPicker ?
          <div className="color-popover">
            <ColorPicker requestClose={this.handleColorClose} colorChange={this.handleColorChange} />
          </div>
          : null
        }
        <input className="add-label-input"
          placeholder="Add a New Label"
          value={this.state.newLabelName}
          onChange={this.handleNameChange} />
      </form>
    );
  }
}

export default AddLabel;
