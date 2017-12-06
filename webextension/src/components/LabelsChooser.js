import React, { Component } from 'react';
import PropTypes from 'prop-types';
import './LabelsChooser.css';

class LabelsChooser extends Component {
  static propTypes = {
    labels: PropTypes.array.isRequired,
    onLabelChecked: PropTypes.func.isRequired,
    onLabelUnchecked: PropTypes.func.isRequired
  }

  render() {
    const { labels, onLabelChecked, onLabelUnchecked } = this.props;
    const onClick = (label) => label.checked ? onLabelUnchecked(label.name):
      onLabelChecked(label.name);
    const labelDivs = labels.map(label =>
      <div className={`label${ label.checked ? ' checked' : ''}`}
        key={label.name}
        onClick={() => onClick(label)}
        style={{ backgroundColor: label.color}}>
        <span className="label-name">{label.name}</span>
        <div className="label-checkmark">✓</div>
      </div>
    );
    return (
      <div className="labels">
        {labelDivs}
      </div>
    );
  }
}

export default LabelsChooser;
