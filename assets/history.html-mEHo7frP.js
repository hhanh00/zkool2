import{_ as n,c as a,a as p,o as t}from"./app-CgU68YD_.js";const e={};function o(l,s){return t(),a("div",null,s[0]||(s[0]=[p(`<div class="language-graphql line-numbers-mode" data-highlighter="prismjs" data-ext="graphql"><pre><code class="language-graphql"><span class="line"><span class="token keyword">query</span> <span class="token punctuation">{</span></span>
<span class="line">  <span class="token property-query">transactionsByAccount</span><span class="token punctuation">(</span><span class="token attr-name">idAccount</span><span class="token punctuation">:</span> <span class="token number">1</span><span class="token punctuation">)</span> <span class="token punctuation">{</span></span>
<span class="line">    <span class="token property">txid</span></span>
<span class="line">    <span class="token property">time</span></span>
<span class="line">    <span class="token property">value</span></span>
<span class="line">    <span class="token object">notes</span> <span class="token punctuation">{</span> <span class="token property">pool</span> <span class="token property">value</span> <span class="token punctuation">}</span></span>
<span class="line">  <span class="token punctuation">}</span></span>
<span class="line"><span class="token punctuation">}</span></span>
<span class="line"></span></code></pre><div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0;"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><div class="language-json line-numbers-mode" data-highlighter="prismjs" data-ext="json"><pre><code class="language-json"><span class="line"><span class="token punctuation">{</span></span>
<span class="line">  <span class="token property">&quot;data&quot;</span><span class="token operator">:</span> <span class="token punctuation">{</span></span>
<span class="line">    <span class="token property">&quot;transactionsByAccount&quot;</span><span class="token operator">:</span> <span class="token punctuation">[</span></span>
<span class="line">      <span class="token punctuation">{</span></span>
<span class="line">        <span class="token property">&quot;txid&quot;</span><span class="token operator">:</span> <span class="token string">&quot;39fbca22ca0bb38cad862d260e0c4589a79d7873ea42756aa80ac89a0b77b99d&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;time&quot;</span><span class="token operator">:</span> <span class="token string">&quot;2026-01-02T16:51:34&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;-0.00010000&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;notes&quot;</span><span class="token operator">:</span> <span class="token punctuation">[</span></span>
<span class="line">          <span class="token punctuation">{</span></span>
<span class="line">            <span class="token property">&quot;pool&quot;</span><span class="token operator">:</span> <span class="token number">2</span><span class="token punctuation">,</span></span>
<span class="line">            <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;0.00035000&quot;</span></span>
<span class="line">          <span class="token punctuation">}</span><span class="token punctuation">,</span></span>
<span class="line">          <span class="token punctuation">{</span></span>
<span class="line">            <span class="token property">&quot;pool&quot;</span><span class="token operator">:</span> <span class="token number">2</span><span class="token punctuation">,</span></span>
<span class="line">            <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;0.00040157&quot;</span></span>
<span class="line">          <span class="token punctuation">}</span></span>
<span class="line">        <span class="token punctuation">]</span></span>
<span class="line">      <span class="token punctuation">}</span><span class="token punctuation">,</span></span>
<span class="line">      <span class="token punctuation">{</span></span>
<span class="line">        <span class="token property">&quot;txid&quot;</span><span class="token operator">:</span> <span class="token string">&quot;224cd18bb829debf623baaa614360caa6dde0e12cca6081f7827f3571b051c7a&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;time&quot;</span><span class="token operator">:</span> <span class="token string">&quot;2026-01-02T16:44:36&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;-0.00010000&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;notes&quot;</span><span class="token operator">:</span> <span class="token punctuation">[</span></span>
<span class="line">          <span class="token punctuation">{</span></span>
<span class="line">            <span class="token property">&quot;pool&quot;</span><span class="token operator">:</span> <span class="token number">2</span><span class="token punctuation">,</span></span>
<span class="line">            <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;0.00040157&quot;</span></span>
<span class="line">          <span class="token punctuation">}</span><span class="token punctuation">,</span></span>
<span class="line">          <span class="token punctuation">{</span></span>
<span class="line">            <span class="token property">&quot;pool&quot;</span><span class="token operator">:</span> <span class="token number">2</span><span class="token punctuation">,</span></span>
<span class="line">            <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;0.00045000&quot;</span></span>
<span class="line">          <span class="token punctuation">}</span></span>
<span class="line">        <span class="token punctuation">]</span></span>
<span class="line">      <span class="token punctuation">}</span><span class="token punctuation">,</span></span>
<span class="line">      <span class="token punctuation">{</span></span>
<span class="line">        <span class="token property">&quot;txid&quot;</span><span class="token operator">:</span> <span class="token string">&quot;773039284130067ec1ec0c4cd8d134a593f30f99b2c54c20c3b76d9ad1566486&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;time&quot;</span><span class="token operator">:</span> <span class="token string">&quot;2026-01-02T16:40:58&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;-0.00010000&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;notes&quot;</span><span class="token operator">:</span> <span class="token punctuation">[</span></span>
<span class="line">          <span class="token punctuation">{</span></span>
<span class="line">            <span class="token property">&quot;pool&quot;</span><span class="token operator">:</span> <span class="token number">2</span><span class="token punctuation">,</span></span>
<span class="line">            <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;0.00040157&quot;</span></span>
<span class="line">          <span class="token punctuation">}</span><span class="token punctuation">,</span></span>
<span class="line">          <span class="token punctuation">{</span></span>
<span class="line">            <span class="token property">&quot;pool&quot;</span><span class="token operator">:</span> <span class="token number">2</span><span class="token punctuation">,</span></span>
<span class="line">            <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;0.00055000&quot;</span></span>
<span class="line">          <span class="token punctuation">}</span></span>
<span class="line">        <span class="token punctuation">]</span></span>
<span class="line">      <span class="token punctuation">}</span><span class="token punctuation">,</span></span>
<span class="line">      <span class="token punctuation">{</span></span>
<span class="line">        <span class="token property">&quot;txid&quot;</span><span class="token operator">:</span> <span class="token string">&quot;08d162c85e9f3e56e1fe30bce8a2aa08e7c83200d079817f9b417d5fde2afda2&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;time&quot;</span><span class="token operator">:</span> <span class="token string">&quot;2026-01-02T16:36:49&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;-0.00010000&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;notes&quot;</span><span class="token operator">:</span> <span class="token punctuation">[</span></span>
<span class="line">          <span class="token punctuation">{</span></span>
<span class="line">            <span class="token property">&quot;pool&quot;</span><span class="token operator">:</span> <span class="token number">2</span><span class="token punctuation">,</span></span>
<span class="line">            <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;0.00040157&quot;</span></span>
<span class="line">          <span class="token punctuation">}</span><span class="token punctuation">,</span></span>
<span class="line">          <span class="token punctuation">{</span></span>
<span class="line">            <span class="token property">&quot;pool&quot;</span><span class="token operator">:</span> <span class="token number">2</span><span class="token punctuation">,</span></span>
<span class="line">            <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;0.00065000&quot;</span></span>
<span class="line">          <span class="token punctuation">}</span></span>
<span class="line">        <span class="token punctuation">]</span></span>
<span class="line">      <span class="token punctuation">}</span><span class="token punctuation">,</span></span>
<span class="line">      <span class="token punctuation">{</span></span>
<span class="line">        <span class="token property">&quot;txid&quot;</span><span class="token operator">:</span> <span class="token string">&quot;4975f4c818f8c33c128423b426075e697aa839036db5af346e2183ca4dab7ea2&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;time&quot;</span><span class="token operator">:</span> <span class="token string">&quot;2026-01-02T12:53:55&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;-0.00010000&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;notes&quot;</span><span class="token operator">:</span> <span class="token punctuation">[</span></span>
<span class="line">          <span class="token punctuation">{</span></span>
<span class="line">            <span class="token property">&quot;pool&quot;</span><span class="token operator">:</span> <span class="token number">2</span><span class="token punctuation">,</span></span>
<span class="line">            <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;0.00040157&quot;</span></span>
<span class="line">          <span class="token punctuation">}</span><span class="token punctuation">,</span></span>
<span class="line">          <span class="token punctuation">{</span></span>
<span class="line">            <span class="token property">&quot;pool&quot;</span><span class="token operator">:</span> <span class="token number">2</span><span class="token punctuation">,</span></span>
<span class="line">            <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;0.00075000&quot;</span></span>
<span class="line">          <span class="token punctuation">}</span></span>
<span class="line">        <span class="token punctuation">]</span></span>
<span class="line">      <span class="token punctuation">}</span><span class="token punctuation">,</span></span>
<span class="line">      <span class="token punctuation">{</span></span>
<span class="line">        <span class="token property">&quot;txid&quot;</span><span class="token operator">:</span> <span class="token string">&quot;7e448c44e8610d451e18e3cc88b8c46b07637a8e2fc6dd5b80690074b061ef8d&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;time&quot;</span><span class="token operator">:</span> <span class="token string">&quot;2026-01-02T12:49:27&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;-0.00020000&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;notes&quot;</span><span class="token operator">:</span> <span class="token punctuation">[</span></span>
<span class="line">          <span class="token punctuation">{</span></span>
<span class="line">            <span class="token property">&quot;pool&quot;</span><span class="token operator">:</span> <span class="token number">2</span><span class="token punctuation">,</span></span>
<span class="line">            <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;0.00040157&quot;</span></span>
<span class="line">          <span class="token punctuation">}</span><span class="token punctuation">,</span></span>
<span class="line">          <span class="token punctuation">{</span></span>
<span class="line">            <span class="token property">&quot;pool&quot;</span><span class="token operator">:</span> <span class="token number">2</span><span class="token punctuation">,</span></span>
<span class="line">            <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;0.00085000&quot;</span></span>
<span class="line">          <span class="token punctuation">}</span></span>
<span class="line">        <span class="token punctuation">]</span></span>
<span class="line">      <span class="token punctuation">}</span><span class="token punctuation">,</span></span>
<span class="line">      <span class="token punctuation">{</span></span>
<span class="line">        <span class="token property">&quot;txid&quot;</span><span class="token operator">:</span> <span class="token string">&quot;587e2f83b3b1a2f7e7bf54aedd548f19b3d1cb10a9c0b806e18ce17bf7654008&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;time&quot;</span><span class="token operator">:</span> <span class="token string">&quot;2026-01-02T12:35:27&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;-0.00015000&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;notes&quot;</span><span class="token operator">:</span> <span class="token punctuation">[</span></span>
<span class="line">          <span class="token punctuation">{</span></span>
<span class="line">            <span class="token property">&quot;pool&quot;</span><span class="token operator">:</span> <span class="token number">0</span><span class="token punctuation">,</span></span>
<span class="line">            <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;0.00040157&quot;</span></span>
<span class="line">          <span class="token punctuation">}</span><span class="token punctuation">,</span></span>
<span class="line">          <span class="token punctuation">{</span></span>
<span class="line">            <span class="token property">&quot;pool&quot;</span><span class="token operator">:</span> <span class="token number">2</span><span class="token punctuation">,</span></span>
<span class="line">            <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;0.00014843&quot;</span></span>
<span class="line">          <span class="token punctuation">}</span></span>
<span class="line">        <span class="token punctuation">]</span></span>
<span class="line">      <span class="token punctuation">}</span><span class="token punctuation">,</span></span>
<span class="line">      <span class="token punctuation">{</span></span>
<span class="line">        <span class="token property">&quot;txid&quot;</span><span class="token operator">:</span> <span class="token string">&quot;8cf7fd5e98db790fec62fe1e740225380f31e71d7512c1cf96c5b6ed67ca7fec&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;time&quot;</span><span class="token operator">:</span> <span class="token string">&quot;2026-01-01T16:14:13&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;-0.00015000&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;notes&quot;</span><span class="token operator">:</span> <span class="token punctuation">[</span></span>
<span class="line">          <span class="token punctuation">{</span></span>
<span class="line">            <span class="token property">&quot;pool&quot;</span><span class="token operator">:</span> <span class="token number">2</span><span class="token punctuation">,</span></span>
<span class="line">            <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;0.00070000&quot;</span></span>
<span class="line">          <span class="token punctuation">}</span><span class="token punctuation">,</span></span>
<span class="line">          <span class="token punctuation">{</span></span>
<span class="line">            <span class="token property">&quot;pool&quot;</span><span class="token operator">:</span> <span class="token number">0</span><span class="token punctuation">,</span></span>
<span class="line">            <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;0.00090157&quot;</span></span>
<span class="line">          <span class="token punctuation">}</span></span>
<span class="line">        <span class="token punctuation">]</span></span>
<span class="line">      <span class="token punctuation">}</span><span class="token punctuation">,</span></span>
<span class="line">      <span class="token punctuation">{</span></span>
<span class="line">        <span class="token property">&quot;txid&quot;</span><span class="token operator">:</span> <span class="token string">&quot;ee5fc4ccbc7b586ea814016ce588fa13b48d71185033c8b57d1f96b869f70e7a&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;time&quot;</span><span class="token operator">:</span> <span class="token string">&quot;2026-01-01T16:00:10&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;-0.00015000&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;notes&quot;</span><span class="token operator">:</span> <span class="token punctuation">[</span></span>
<span class="line">          <span class="token punctuation">{</span></span>
<span class="line">            <span class="token property">&quot;pool&quot;</span><span class="token operator">:</span> <span class="token number">0</span><span class="token punctuation">,</span></span>
<span class="line">            <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;0.00090157&quot;</span></span>
<span class="line">          <span class="token punctuation">}</span><span class="token punctuation">,</span></span>
<span class="line">          <span class="token punctuation">{</span></span>
<span class="line">            <span class="token property">&quot;pool&quot;</span><span class="token operator">:</span> <span class="token number">2</span><span class="token punctuation">,</span></span>
<span class="line">            <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;0.00085000&quot;</span></span>
<span class="line">          <span class="token punctuation">}</span></span>
<span class="line">        <span class="token punctuation">]</span></span>
<span class="line">      <span class="token punctuation">}</span><span class="token punctuation">,</span></span>
<span class="line">      <span class="token punctuation">{</span></span>
<span class="line">        <span class="token property">&quot;txid&quot;</span><span class="token operator">:</span> <span class="token string">&quot;9ec52927528d03b5840796760096ce43c53f1e071b79431a813643595d185327&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;time&quot;</span><span class="token operator">:</span> <span class="token string">&quot;2026-01-01T14:41:33&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;0.00190157&quot;</span><span class="token punctuation">,</span></span>
<span class="line">        <span class="token property">&quot;notes&quot;</span><span class="token operator">:</span> <span class="token punctuation">[</span></span>
<span class="line">          <span class="token punctuation">{</span></span>
<span class="line">            <span class="token property">&quot;pool&quot;</span><span class="token operator">:</span> <span class="token number">2</span><span class="token punctuation">,</span></span>
<span class="line">            <span class="token property">&quot;value&quot;</span><span class="token operator">:</span> <span class="token string">&quot;0.00190157&quot;</span></span>
<span class="line">          <span class="token punctuation">}</span></span>
<span class="line">        <span class="token punctuation">]</span></span>
<span class="line">      <span class="token punctuation">}</span></span>
<span class="line">    <span class="token punctuation">]</span></span>
<span class="line">  <span class="token punctuation">}</span></span>
<span class="line"><span class="token punctuation">}</span></span>
<span class="line"></span></code></pre><div class="line-numbers" aria-hidden="true" style="counter-reset:line-number 0;"><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div><div class="line-number"></div></div></div><div class="hint-container info"><p class="hint-container-title">Info</p><p>Transactions can be filtered by height.</p></div>`,3)]))}const i=n(e,[["render",o]]),u=JSON.parse('{"path":"/graphql/history.html","title":"Transaction History","lang":"en-US","frontmatter":{"title":"Transaction History"},"git":{"updatedTime":1767805130000,"contributors":[{"name":"Hanh","username":"Hanh","email":"hanh425@gmail.com","commits":1,"url":"https://github.com/Hanh"}],"changelog":[{"hash":"553c4f6cac510eee3c3a55323875f1e9ddccff25","time":1767805130000,"email":"hanh425@gmail.com","author":"Hanh","message":"Include instructions for using testnet and regtest"}]},"filePathRelative":"graphql/history.md"}');export{i as comp,u as data};
